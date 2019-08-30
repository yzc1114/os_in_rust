#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]
#![cfg_attr(test, allow(unused_imports))]
#![feature(alloc)]
#![feature(asm)]

#[macro_use]
extern crate ryzc;
extern crate alloc;

use alloc::string::String;
use bootloader::{bootinfo::BootInfo, entry_point};
use core::panic::PanicInfo;
use ryzc::{arch, device, syscall, task, console, fs};
use ryzc::arch::memory::stack_allocator;

entry_point!(kernel_main);


#[cfg(not(test))]
fn kernel_main(boot_info: &'static BootInfo) -> ! {

    use arch::memory::heap::{HEAP_SIZE, HEAP_START};

    unsafe {
        arch::init(boot_info);
        task::scheduler::init();
        arch::interrupts::clear_mask();
        fs::init();
    }

    
    //kprintln!("\nHEAP START = 0x{:x}", HEAP_START);
    //kprintln!("HEAP END = 0x{:x}\n", HEAP_START + HEAP_SIZE);
    //kprintln!("bootinfo : {:?}", boot_info);
    let _ = syscall::create(String::from("console"), 0, console::console_process);
    //let _ = syscall::create(String::from("test_process"), 1, ryzc::test_process);
    loop {
        // #[cfg(feature = "serial")]
        // {
        //     use device::uart_16550 as uart;
        //     uart::read(1024);
        // }

        // Save cycles by pausing until next interrupt
        arch::interrupts::pause();
        
    }
}

#[cfg(not(test))]
#[panic_handler]
pub fn panic(info: &PanicInfo) -> ! {
    kprintln!("{}", info);

    loop {
        unsafe {
            arch::interrupts::halt();
        }
    }
}
