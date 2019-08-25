use crate::fs;
use crate::device::hard_disk;
use core::convert::TryInto;
use core::iter::FromIterator;
use alloc::vec::Vec;
use alloc::string::String;


pub struct BlockDevice;

pub const META_INFO_PER_BLOCK: usize = 16;

impl BlockDevice{
    pub fn read_block(block_index: usize) -> Result<Vec::<u8>, ()>{
        assert!(block_index < fs::BLOCK_NUM);
        let bufu32: &mut [u32] = &mut [0; fs::BLOCK_SIZE / 4];
        //kprintln!("before read");
        let result = fs::HDB.read(block_index as u64, 1, bufu32);
        //kprintln!("after read");
        let ptr :*const u32 = bufu32.as_ptr();
        let ptr :*const u8 = ptr as * const u8;
        match result {
            Ok(_) => {
                //kprintln!("result ok !!!");
                let mut v = Vec::new();
                for i in 0..fs::BLOCK_SIZE{
                    v.push( unsafe { *(ptr.offset((i as usize).try_into().unwrap())) } );
                }
                Ok(v)
            },
            Err(_) => {
                Err(())
            }
        }
    }
    pub fn write_block(block_index: usize, buf: &mut [u8]) -> Result<(), ()>{
        assert!(block_index < fs::BLOCK_NUM);
        assert_eq!(buf.len(), fs::BLOCK_SIZE);
        let bufu32: &mut [u32] = &mut [0; fs::BLOCK_SIZE / 4];
        let ptr :*const u8 = buf.as_ptr();
        let ptr :*const u32 = ptr as *const u32;
        for i in 0..bufu32.len(){
            bufu32[i] = unsafe { *(ptr.offset((i as usize).try_into().unwrap()) ) };
        }
        fs::HDB.write(block_index as u64, 1, bufu32)

    }
    pub fn clear_block(block_index: usize) -> Result<(), ()>{
        assert!(block_index < fs::BLOCK_NUM);
        let buf: &mut [u32] = &mut [0; fs::BLOCK_SIZE / 4];
        fs::HDB.write(block_index as u64, 1, buf)
    }

    pub fn init() -> Vec<fs::file::File>{
        let files = Vec::new();
        //前128个块中包含全部文件信息。
        let super_blocks_num = fs::SUPER_BLOCKS_NUM;
        //这其中最后一个区块用于空闲区管理
        let block_vain_index = fs::UNUSED_BLOCKS_BITMAP_INDEX;
        

        return files;
    }

    pub fn get_files() -> Vec<fs::file::File> {
        let mut files = Vec::new();
        //前256个块中包含全部文件信息。
        let super_blocks_num = fs::SUPER_BLOCKS_NUM;
        //这其中最后一个区块用于空闲区管理
        let last_block = match BlockDevice::read_block(super_blocks_num - 1) {
            Ok(v) => Some(v),
            Err(_) => None
        };
        for i in 0..(super_blocks_num-1){
            match BlockDevice::read_block(i){
                Err(_) => panic!("read block err"),
                Ok(v) => {
                    //每个block存放16个文件的元信息。
                    for j in 0..16{
                        let file_meta_info: &[u8] = &v[j..j+(fs::BLOCK_SIZE / META_INFO_PER_BLOCK)];
                        let file = match BlockDevice::parse_file_meta_info(file_meta_info){
                            Some(file) => file,
                            None => continue
                        };
                        files.push(file);
                    }
                }
            }
        }
        //TODO: 处理空闲区块
        return files;
    }

    fn parse_file_meta_info(meta: &[u8]) -> Option<fs::file::File> {
        //meta的长度应为 BLOCK_SIZE / 16
        assert_eq!(meta.len(), fs::BLOCK_SIZE / 16);
        let block_index: usize;
        let block_count: usize;
        let u8_length;
        //前16位为块起始索引
        //前17-32位为块的数量
        block_index = ((meta[0] as usize) << 8 | meta[1] as usize) as usize;
        block_count = ((meta[2] as usize) << 8 | meta[3] as usize) as usize;
        u8_length = ((meta[4] as usize) << 8 | meta[5] as usize) as usize;
        if block_index <= fs::SUPER_BLOCKS_NUM {
            return None;
        }
        let mut name = String::new();
        for c in 4..meta.len(){
            name.push(meta[c] as char);
        }
        Some(fs::file::File{
            block_index,
            name,
            block_count,
            u8_length,
        })
    }
}

pub fn transfrom_to_u32_array(buf: &[u8], bufu32: &mut [u32]){
    assert_eq!(buf.len(), bufu32.len() * 4);

    let ptr :*const u8 = buf.as_ptr();
    let ptr :*const u32 = ptr as * const u32;
    let bufu32ptr = bufu32.as_mut_ptr();
    unsafe {*bufu32ptr = *ptr}
}

pub fn transfrom_to_u8_array(buf: &mut [u8], bufu32: &[u32]){
    assert_eq!(buf.len(), bufu32.len() * 4);

    let ptr :*const u32 = bufu32.as_ptr();
    let ptr :*const u8 = ptr as *const u8;
    let bufu8ptr = buf.as_mut_ptr();
    unsafe {*bufu8ptr = *ptr}
}

