pub mod vfs;
pub mod file;
pub mod block_device;


pub use vfs::*;
pub use file::*;
pub use block_device::*;


use crate::device::hard_disk;
use crate::device::hard_disk::*;
use core::convert::TryInto;
use alloc::vec::Vec;
use alloc::string::String;
use core::iter::Iterator;

pub const BLOCK_NUM : usize = 4096;
pub const SUPER_BLOCKS_NUM : usize = 255;
pub const UNUSED_BLOCKS_BITMAP_INDEX : usize = 255;
pub const BLOCK_SIZE : usize = 512;


lazy_static!{
    pub static ref HDB: hard_disk::IDE = {hard_disk::IDE::new(1)};
}

pub fn init(){
    let hdb_num = HDB.num;
    //kprintln!("hdb: num{}", hdb_num);
    let hdb_base = HDB.base;
    //kprintln!("hdb: base{}", hdb_base);
    let buf: &mut [u8] = &mut [0; BLOCK_SIZE];
    for i in 0..32{
        buf[i] = 0xFF as u8;
    }

    refresh_disk();

    match BlockDevice::write_block(255, buf) {
        Ok(_) => (),//kprintln!("write ok"),
        Err(_) => (),//kprintln!("write err")
    }

    //kprintln!("unused_blocks: {:?}", block_device::BlockDevice::get_unused_blocks());

    let content = String::from("test short file");
    match File::create_file(String::from("myfile"), &content) {
        Ok(f) => {
            //kprintln!("ok, f: {:?}, content:{:?}",f, f.read_content().unwrap())
        },
        Err(_) => kprintln!("no!"),
    }

    let content = String::from("If by Life You Were Deceived:
If by life you were deceived,
Don't be dismal,don't be wild!
In the day of grief,be mild:
Merry days will come,believe.
Heart is living in tomorrow;
Present is dejected here:
In a moment,passes sorrow;
That which passes will be dear.");
    match File::create_file(String::from("poem"), &content) {
        Ok(f) => {
            //kprintln!("ok, file content: {}, f: {:?}, ", f.read_content().unwrap(), f);
        },
        Err(_) => kprintln!("no!"),
    }

    //kprintln!("files: {}", files.len());
    //kprintln!("files: {:?}", files);

    // let mut v = Vec::new();
    // v.push(1 as u8);
    // let bmap = block_device::BitMap::new(v);
    // if bmap.check_exists(7){
    //     kprintln!("exist!");
    // }else{
    //     kprintln!("no exist!");
    // }

    // match BlockDevice::read_block(1024) {
    //     Ok(v) => kprintln!("read ok, {:?}", v),
    //     Err(_) => kprintln!("read err")
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