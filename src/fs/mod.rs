pub mod vfs;
pub mod file;
pub mod block_device;


pub use vfs::*;
pub use file::*;
pub use block_device::*;

use crate::device::hard_disk;
use crate::device::hard_disk::*;
use core::convert::TryInto;

pub const BLOCK_NUM : usize = 4096;
pub const SUPER_BLOCKS_NUM : usize = 255;
pub const UNUSED_BLOCKS_BITMAP_INDEX : usize = 255;
pub const BLOCK_SIZE : usize = 512;


lazy_static!{
    pub static ref HDB: hard_disk::IDE = {hard_disk::IDE::new(1)};
}

pub fn init(){
    let hdb_num = HDB.num;
    kprintln!("hdb: num{}", hdb_num);
    let hdb_base = HDB.base;
    kprintln!("hdb: base{}", hdb_base);
    let buf: &mut [u8] = &mut [0; BLOCK_SIZE];
    let buf32: &mut [u32] = &mut [0; SECTOR_SIZE];
    for i in 0..buf32.len(){
        buf32[i] = i as u32;
    }
    for i in 0..buf.len(){
        buf[i] = i as u8;
    }

    //refresh_disk();

    // match BlockDevice::read_block(4093) {
    //     Ok(v) => kprintln!("read ok, {:?}", v),
    //     Err(_) => kprintln!("read err")
    // }

    // match BlockDevice::write_block(4093, buf) {
    //     Ok(_) => kprintln!("write ok"),
    //     Err(_) => kprintln!("write err")
    // }

    // let buf: &mut [u32] = &mut [0; SECTOR_SIZE];

    //kprintln!("hdb: num:{}, base:{}, ctrl:{}", HDB.num, HDB.base, HDB.ctrl);
}


pub fn refresh_disk(){
    let buf: &mut [u32] = &mut [0; 128];
    for i in 0..BLOCK_NUM{
        match HDB.write((i as usize).try_into().unwrap(), 1, buf){
            Ok(_) => continue,
            Err(_) => kprintln!("write disk err")
        }
    }
}