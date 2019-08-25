use crate::device::{keyboard, pic_8259, pit, ps2_controller_8042, uart_16550, hard_disk};


pub fn init() {
    pic_8259::init();
    uart_16550::init();
    ps2_controller_8042::init();
    pit::init();
    keyboard::ps2::init();
    //kprintln!("hdb: name:{}, base:{}, ctrl:{}", hdb.num, hdb.base, hdb.ctrl);
}
