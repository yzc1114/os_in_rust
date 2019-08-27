use bootloader::bootinfo::BootInfo;

use crate::HEAP_ALLOCATOR;

pub mod context;
pub mod device;
pub mod idt;
pub mod interrupts;
pub mod memory;
pub mod io;
pub mod syscall;

pub unsafe fn init(boot_info: &'static BootInfo) {
    for region in boot_info.memory_map.iter() {
        kprintln!("{:?}", region);
    }
    //kprintln!("memory init");
    memory::init(boot_info);

    use self::memory::heap::{HEAP_SIZE, HEAP_START};
    HEAP_ALLOCATOR.init(HEAP_START as usize, HEAP_SIZE as usize);

    idt::init();
    device::init();
}
