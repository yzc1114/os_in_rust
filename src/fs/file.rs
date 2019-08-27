use crate::fs;
use crate::fs::block_device;
use alloc::string::String;
use alloc::vec::Vec;
use core::convert::TryInto;

#[derive(core::fmt::Debug)]
pub struct File {
    pub block_index: usize,
    pub name: String,
    pub block_count: usize,
    pub u8_length: usize,
}

impl File {
    pub fn read_content(&self) -> Result<String, ()> {
        let mut content = String::new();
        for i in 0..self.block_count {
            if i < self.block_count - 1 {
                match block_device::BlockDevice::read_block(self.block_index + i) {
                    Ok(v) => {
                        // if i == 4{
                        //     kprintln!("{:?}", v);
                        // }
                        content.push_str(
                            &mut v.iter().map(|u| u.clone() as char).collect::<String>(),
                        );
                    }
                    Err(_) => return Err(()),
                }
            } else {
                match block_device::BlockDevice::read_block(self.block_index + self.block_count - 1) {
                    Ok(v) => {
                        //kprintln!("read block:{:?}", v);
                        let mut temp = v.iter().map(|u| u.clone() as char).collect::<String>();
                        temp.truncate(self.u8_length - (self.block_count - 1) * fs::BLOCK_SIZE);
                        //kprintln!("{}" ,temp);
                        content.push_str(&mut temp);
                        //let v2be_append = Vec::from(t[0..(fs::BLOCK_SIZE - (self.block_count * fs::BLOCK_SIZE - self.u8_length)]);
                        //content.append(Vec::from(v.iter().map(|u| u.clone() as char).collect::<Vec::<char>>()[0..(fs::BLOCK_SIZE - (self.block_count * fs::BLOCK_SIZE - self.u8_length)]))
                    }
                    Err(_) => return Err(()),
                }
            }
        }
        return Ok(content);
    }

    pub fn rename(&self, new_name: &String) -> Result<(), ()> {
        fs::BlockDevice::change_name(&self.name, new_name)
    }

    pub fn remove(self) -> Result<(), ()> {
        fs::BlockDevice::remove_file(&self.name)
    }

}
