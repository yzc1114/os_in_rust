use crate::arch::interrupts;
use crate::syscall::error::Error;
use crate::task::scheduler::{global_sched, Scheduling};
use crate::task::{ProcessId, State};
use alloc::string::String;

/// Wrapper around scheduler.create() and ready() that can be called in processes
pub fn create(name: String, prio: usize, proc_entry: extern "C" fn()) -> Result<ProcessId, Error> {
    let pid = global_sched()
        .create(name, prio, proc_entry)
        .expect("Could not create new process!");
    global_sched().ready(pid)?;
    Ok(pid)
}

/// Wrapper around scheduler.kill()
pub fn kill(pid: ProcessId) -> Result<(), Error> {
    global_sched().kill(pid)?;
    Ok(())
}

/// Ready a suspended process
pub fn resume(pid: ProcessId) -> Result<(), Error> {
    let proc_ref = global_sched().get_process(pid)?;
    if proc_ref.read().state == State::Suspended {
        global_sched().ready(pid)?;
        Ok(())
    } else {
        Err(Error::InvalidOperation)
    }
}

/// Suspend a ready or currently running process
pub fn suspend(pid: ProcessId) -> Result<(), Error> {
    let proc_ref = global_sched().get_process(pid)?;

    if pid == ProcessId::NULL_PROCESS {
        return Err(Error::InvalidOperation);
    }

    interrupts::disable_then_execute(|| {
        let state = proc_ref.read().state;
        match state {
            State::Ready => {
                global_sched().modify_process(pid, |proc_ref| {
                    proc_ref.write().set_state(State::Suspended);
                })?;

                global_sched().unready(pid)?;
            }

            State::Current => {
                global_sched().modify_process(pid, |proc_ref| {
                    proc_ref.write().set_state(State::Suspended);
                })?;

                unsafe {
                    global_sched().resched()?;
                }
            }

            _ => return Err(Error::InvalidOperation),
        }

        Ok(())
    })
}

/// Donate CPU time slice to another process
pub fn yield_cpu() -> Result<(), Error> {
    unsafe {
        global_sched().resched()?;
    }

    Ok(())
}


pub fn get_curr_pid() -> ProcessId{
    unsafe{
        global_sched().get_pid()
    }
}

pub fn sleep(ticks: u64){
    let init_tick = unsafe { *crate::arch::tick.lock() };
    //kprintln!("init tick: {}", init_tick);
    loop{
        {
            let curr_tick = unsafe { *crate::arch::tick.lock() };
            //kprintln!("curr tick {}", curr_tick);
            if curr_tick - init_tick > ticks{
                return;
            }
        }
        match yield_cpu() {
            Ok(_) => continue,
            Err(_) => return
        }
    }
}

pub fn wait(pid: ProcessId){
    //wait for pid to exit
    let proc_ref = match global_sched().get_process(pid){
        Ok(r) => r,
        Err(_) => return
    };
    loop{
        let state = proc_ref.read().state;
        match state{
            State::Free => return,
            _ => yield_cpu().unwrap_or_default()
        }
    }
}

pub fn get_ticks() -> u64{
    let ticks = unsafe { crate::arch::tick.lock() };
    *ticks
}