#![feature(alloc)]
#![feature(const_fn)]
#![no_std] // don't link the Rust standard library
#![cfg_attr(not(test), no_main)] // disable all Rust-level entry points
#![cfg_attr(test, allow(dead_code, unused_macros, unused_imports))]

#[macro_use]
extern crate ryzc;

extern crate alloc;
extern crate spin;

use alloc::string::String;
use bootloader::bootinfo::BootInfo;
use core::panic::PanicInfo;
use core::sync::atomic::{AtomicBool, Ordering};
use ryzc::exit_qemu;

static SUSPENDED_PROC: AtomicBool = AtomicBool::new(false);

#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn _start(boot_info: &'static BootInfo) -> ! {
    use ryzc::syscall;

    unsafe {
        ryzc::arch::init(boot_info);
        ryzc::task::scheduler::init();
    }

    let kill_pid = syscall::create(String::from("kill"), 0, kill_process).unwrap();
    match syscall::kill(kill_pid) {
        Ok(()) => (),
        _ => {
            serial_println!("failed");
            serial_println!("Failed to kill process");
        }
    }

    let suspend_pid = syscall::create(String::from("suspend"), 0, suspend_process).unwrap();
    match syscall::suspend(suspend_pid) {
        Ok(()) => (),
        _ => {
            serial_println!("failed");
            serial_println!("Failed to suspend process");
        }
    }

    SUSPENDED_PROC.store(true, Ordering::SeqCst);

    let _ = syscall::create(String::from("loop"), 0, loop_process).unwrap();
    let _ = syscall::yield_cpu().unwrap();

    match syscall::resume(suspend_pid) {
        Ok(()) => (),
        _ => {
            serial_println!("failed");
            serial_println!("Failed to resume process");
        }
    }

    let _ = syscall::yield_cpu().unwrap();

    serial_println!("failed");
    serial_println!("'suspend' process did not run!");
    unsafe {
        exit_qemu();
    }

    loop {}
}

pub extern "C" fn kill_process() {
    serial_println!("failed");
    unsafe {
        exit_qemu();
    }
}

/// Test repeated calls to `yield_cpu()`
pub extern "C" fn loop_process() {
    loop {
        let _ = ryzc::syscall::yield_cpu().unwrap();
    }
}

pub extern "C" fn suspend_process() {
    assert!(SUSPENDED_PROC.load(Ordering::SeqCst));

    serial_println!("ok");
    unsafe {
        exit_qemu();
    }
}

#[cfg(not(test))]
#[panic_handler]
#[no_mangle]
pub fn panic(info: &PanicInfo) -> ! {
    serial_println!("failed");

    serial_println!("{}", info);
    unsafe {
        exit_qemu();
    }
    loop {}
}
