use crate::arch::x86_64::interrupts::{exception, irq};
use crate::arch::x86_64::interrupts::syscall;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::idt::InterruptDescriptorTable;
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;
use x86_64::structures::idt::*;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

lazy_static! {
    pub static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };
        tss
    };
    pub static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));

        let selectors = Selectors {
            code_selector,
            tss_selector,
        };

        (gdt, selectors)
    };
}

pub struct Selectors {
    code_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

const IRQ_OFFSET: usize = 32;
#[allow(dead_code)]
const SYSCALL_OFFSET: usize = 0x80;

lazy_static! {
    static ref IDT: InterruptDescriptorTable  = {
        let mut idt = InterruptDescriptorTable::new();

        idt.divide_by_zero.set_handler_fn(exception::divide_by_zero);
        idt.debug.set_handler_fn(exception::debug);
        idt.non_maskable_interrupt.set_handler_fn(exception::non_maskable_interrupt);
        idt.breakpoint.set_handler_fn(exception::breakpoint);
        idt.overflow.set_handler_fn(exception::overflow);
        idt.bound_range_exceeded.set_handler_fn(exception::bound_range_exceeded);
        idt.invalid_opcode.set_handler_fn(exception::invalid_opcode);
        idt.device_not_available.set_handler_fn(exception::device_not_available);
        unsafe {
            idt.double_fault.set_handler_fn(exception::double_fault)
                .set_stack_index(DOUBLE_FAULT_IST_INDEX);
        }
        idt.invalid_tss.set_handler_fn(exception::invalid_tss);
        idt.segment_not_present.set_handler_fn(exception::segment_not_present);
        idt.stack_segment_fault.set_handler_fn(exception::stack_segment_fault);
        idt.general_protection_fault.set_handler_fn(exception::general_protection_fault);
        idt.page_fault.set_handler_fn(exception::page_fault);
        idt.x87_floating_point.set_handler_fn(exception::x87_floating_point);
        idt.alignment_check.set_handler_fn(exception::alignment_check);
        idt.machine_check.set_handler_fn(exception::machine_check);
        idt.simd_floating_point.set_handler_fn(exception::simd_floating_point);
        idt.virtualization.set_handler_fn(exception::virtualization);
        idt.security_exception.set_handler_fn(exception::security_exception);

        idt[IRQ_OFFSET + 0].set_handler_fn(irq::timer);
        idt[IRQ_OFFSET + 1].set_handler_fn(irq::keyboard);
        idt[IRQ_OFFSET + 2].set_handler_fn(irq::cascade);
        idt[IRQ_OFFSET + 3].set_handler_fn(irq::com2);
        idt[IRQ_OFFSET + 4].set_handler_fn(irq::com1);
        idt[IRQ_OFFSET + 14].set_handler_fn(irq::ide);

        // TODO: Syscall
        //idt[SYSCALL_OFFSET] = syscall_handler_entry(syscall::syscall);
        idt[SYSCALL_OFFSET].set_handler_fn(syscall::syscall);

        idt
    };
}


pub fn init() {
    use x86_64::instructions::segmentation::set_cs;
    use x86_64::instructions::tables::load_tss;
    // unsafe {
	// 	wrmsr(IA32_EFER, rdmsr(IA32_EFER) | EFER_LMA | EFER_SCE | EFER_NXE);
	// 	wrmsr(IA32_STAR, (0x1Bu64 << 48) | (0x08u64 << 32));
	// 	wrmsr(IA32_LSTAR, syscall_handler as u64);
	// 	wrmsr(IA32_FMASK, 1 << 9); // clear IF flag during system call

	// 	// reset GS registers
	// 	wrmsr(IA32_KERNEL_GS_BASE, 0);
	// 	asm!("wrgsbase $0" :: "r"(BOOT_STACK.top()) :: "volatile");
	// }

    GDT.0.load();

    unsafe {
        set_cs(GDT.1.code_selector);
        load_tss(GDT.1.tss_selector);
    }

    IDT.load();
}



// pub unsafe fn wrmsr(msr: u32, value: u64) {
//     let low = value as u32;
//     let high = (value >> 32) as u32;
//     asm!("wrmsr" :: "{ecx}" (msr), "{eax}" (low), "{edx}" (high) : "memory" : "volatile" );
// }

// /// Read 64 bits msr register.
// #[allow(unused_mut)]
// pub unsafe fn rdmsr(msr: u32) -> u64 {
//     let (high, low): (u32, u32);
//     asm!("rdmsr" : "={eax}" (low), "={edx}" (high) : "{ecx}" (msr) : "memory" : "volatile");
//     ((high as u64) << 32) | (low as u64)
// }

// pub const IA32_EFER: u32 = 0xc0000080;
// pub const IA32_STAR: u32 = 0xc0000081;
// pub const IA32_LSTAR: u32 = 0xc0000082;
// pub const IA32_FMASK: u32 = 0xc0000084;
// pub const IA32_KERNEL_GS_BASE: u32 = 0xc0000102;
// const EFER_SCE: u64 = (1 << 0);
// const EFER_LME: u64 = (1 << 8);
// const EFER_LMA: u64 = (1 << 10);
// const EFER_NXE: u64 = (1 << 11);
// const EFER_SVME: u64 = (1 << 12);
// const EFER_LMSLE: u64 = (1 << 13);
// const EFER_FFXSR: u64 = (1 << 14);
// const EFER_TCE: u64 = (1 << 15);