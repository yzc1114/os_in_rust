#![feature(
    abi_x86_interrupt,
    alloc,
    allocator_api,
    alloc_error_handler,
    asm,
    const_fn,
    global_asm,
    lang_items,
    naked_functions,
    ptr_internals,
    const_vec_new
)]
#![no_std]

#[macro_use]
extern crate alloc;

#[macro_use]
extern crate bitflags;

#[macro_use]
extern crate lazy_static;

//#[macro_use]
//extern crate fatfs;

#[cfg(test)]
extern crate array_init;
#[cfg(test)]
extern crate std;

#[macro_use]
pub mod device;

#[macro_use]
pub mod arch;

pub mod sync;
pub mod syscall;
pub mod task;
pub mod console;
pub mod fs;
pub mod demo_task;

pub unsafe fn exit_qemu() {
    use x86_64::instructions::port::Port;

    let mut port = Port::<u32>::new(0xf4);
    port.write(0);
}

#[alloc_error_handler]
pub fn rust_oom(info: core::alloc::Layout) -> ! {
    panic!("{:?}", info);
}

#[lang = "eh_personality"]
#[no_mangle]
pub extern "C" fn eh_personality() {
    loop {}
}

use arch::memory::heap::HeapAllocator;

#[global_allocator]
pub static HEAP_ALLOCATOR: HeapAllocator = HeapAllocator::new();
