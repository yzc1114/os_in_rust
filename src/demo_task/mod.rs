use crate::{arch, console, device, fs, syscall, task};
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use core::panic::PanicInfo;

pub extern "C" fn multi_test_process() {
    //kprintln!("get curr pid");
    let pid = syscall::get_curr_pid();
    //kprintln!("get over");
    for i in 0..20 {
        //kprintln!("about to sleep");
        syscall::sleep(500);
        //kprintln!("sleep over");
        kprintln!("running {:?} {} time", pid, i);
    }
}

pub extern "C" fn test_syscall() {
    let id: usize = 1;
    let (arg0, arg1, arg2, arg3, arg4, arg5) = (1, 2, 3, 4, 5, 6);
    // unsafe {
    //     asm!("int 0x80"
    //         : "={rax}" (ret)
    //         : "{rax}" (id), "{rdi}" (arg0), "{rsi}" (arg1), "{rdx}" (arg2), "{r10}" (arg3), "{r8}" (arg4), "{r9}" (arg5)
    //         : "rcx" "r11" "memory"
    //         : "intel" "volatile");
    // }
    unsafe {
        asm!("push 20;
        pop rax;
        int 0x80;" :::: "intel", "volatile");
    };
    // let ret: u64 = unsafe {
    //     let rax;
    //     asm!("mov %rax, $0" : "=r"(rax));
    //     rax
    // };
    //kprintln!("ret: {}", ret);
}

use crate::sync::IrqLock;
use crate::sync::Semaphore;
lazy_static! {
    static ref empty: IrqLock<Semaphore> = { IrqLock::new(Semaphore::new(10)) };
    static ref full: IrqLock<Semaphore> = { IrqLock::new(Semaphore::new(0)) };
    static ref buffer: IrqLock<Vec::<u8>> = { IrqLock::new(Vec::with_capacity(10)) };
}

static mut produce_over : IrqLock<bool> = IrqLock::new(false);

pub extern "C" fn test_semaphore() {
    let producer = match syscall::create(String::from("producer"), 50, producer) {
        Ok(pid) => pid,
        Err(_) => {
            kprintln!("producer create failed!");
            return;
        }
    };
    let mut consumers = Vec::new();
    for i in 0..5 {
        let i = i as usize;
        let c = match syscall::create(String::from("consumer") + &(i.to_string()), 50, consumer) {
            Ok(pid) => pid,
            Err(_) => {
                kprintln!("consumer create failed!");
                return;
            }
        };
        consumers.push(c);
    }
    kprintln!("all created!");
    syscall::wait(producer);
    while !unsafe { *produce_over.lock() } || buffer.lock().len() != 0 {
        syscall::yield_cpu().unwrap();
        //等待任务结束
    }
    for i in 0..consumers.len(){
        syscall::kill(consumers[i]).unwrap();
    }
}

pub extern "C" fn producer() {
    let products = 20;
    let self_pid = syscall::get_curr_pid();
    for i in 1..=products {
        match empty.lock().wait() {
            Ok(_) => (),
            Err(e) => {
                kprintln!("{:?}", e);
                return;
            }
        }
        {
            buffer.lock().push(i);
        }
        kprintln!("{:?} producing {}", self_pid, i);
        syscall::sleep(300);
        match full.lock().signal() {
            Ok(_) => (),
            Err(e) => {
                kprintln!("{:?}", e);
                return;
            }
        }
    }
    kprintln!("producer over");
    unsafe {
        let mut produce_over_ref = produce_over.lock();
        *produce_over_ref = true;
    }
}

pub extern "C" fn consumer() {
    let self_pid = syscall::get_curr_pid();
    loop {
        match full.lock().wait() {
            Ok(_) => (),
            Err(e) => {
                kprintln!("{:?}", e);
                return;
            }
        }
        let product = { buffer.lock().pop().expect("pop error") };
        kprintln!("{:?} consuming {}", self_pid, product);
        //syscall::sleep(100);
        match empty.lock().signal() {
            Ok(_) => (),
            Err(e) => {
                kprintln!("{:?}", e);
                return;
            }
        }
    }
}
