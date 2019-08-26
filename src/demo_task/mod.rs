use alloc::string::String;
use core::panic::PanicInfo;
use crate::{arch, device, syscall, task, console, fs};

pub extern "C" fn multi_test_process(){
    //kprintln!("get curr pid");
    let pid = syscall::get_curr_pid();
    //kprintln!("get over");
    for i in 0..20{
        //kprintln!("about to sleep");
        syscall::sleep(500);
        //kprintln!("sleep over");
        kprintln!("running {:?} {} time", pid, i);
    }
}

