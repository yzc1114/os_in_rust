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

pub extern "C" fn test_syscall() {
    let id :usize = 1;
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