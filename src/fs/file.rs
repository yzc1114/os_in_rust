use crate::fs::block_device;
use alloc::string::String;
use alloc::vec::Vec;
use core::convert::TryInto;
pub struct File{
    pub block_index: usize,
    pub name: String,
    pub block_count: usize,
    pub u8_length: usize,
}

impl File{
    pub fn read_block(block_index: usize) -> Result<Vec::<u8>, ()>{
        block_device::BlockDevice::read_block(block_index)
    }

    pub fn write_block(block_index: usize, buf : &mut [u8]) -> Result<(), ()>{
        block_device::BlockDevice::write_block(block_index, buf)
    }

    pub fn read_content(&self) -> Result<Vec::<char>, ()> {
        let mut content = Vec::new();
        for i in 0..self.block_count-1 {
            match block_device::BlockDevice::read_block((i as usize).try_into().unwrap()) {
                Ok(v) => {
                    content.append(&mut v.iter().map(|u| u.clone() as char).collect::<Vec::<char>>());
                },
                Err(_) => return Err(())
            }
        }
        match block_device::BlockDevice::read_block(self.block_count - 1) {
            Ok(v) => {
                let t = v.iter().map(|u| u.clone() as char).collect::<Vec::<char>>();
                
                //let v2be_append = Vec::from(t[0..(fs::BLOCK_SIZE - (self.block_count * fs::BLOCK_SIZE - self.u8_length)]);
                //content.append(Vec::from(v.iter().map(|u| u.clone() as char).collect::<Vec::<char>>()[0..(fs::BLOCK_SIZE - (self.block_count * fs::BLOCK_SIZE - self.u8_length)]))
            },
            Err(_) => return Err(())
        }
        return Ok(content);
    }
}
