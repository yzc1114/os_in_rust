use crate::fs;
use alloc::string::String;
use alloc::vec::Vec;
use core::convert::TryInto;

pub struct BlockDevice;

pub const META_INFO_PER_BLOCK: usize = 16;
pub const META_INFO_LEN: usize = fs::BLOCK_SIZE / META_INFO_PER_BLOCK;

impl BlockDevice {
    pub fn read_block(block_index: usize) -> Result<Vec<u8>, ()> {
        assert!(block_index < fs::BLOCK_NUM);
        let bufu32: &mut [u32] = &mut [0; fs::BLOCK_SIZE / 4];
        //kprintln!("before read");
        let result = fs::HDB.read(block_index as u64, 1, bufu32);
        //kprintln!("after read");
        let ptr: *const u32 = bufu32.as_ptr();
        let ptr: *const u8 = ptr as *const u8;
        match result {
            Ok(_) => {
                //kprintln!("result ok !!!");
                let mut v = Vec::new();
                for i in 0..fs::BLOCK_SIZE {
                    v.push(unsafe { *(ptr.offset((i as usize).try_into().unwrap())) });
                }
                Ok(v)
            }
            Err(_) => Err(()),
        }
    }
    pub fn write_block(block_index: usize, buf: &[u8]) -> Result<(), ()> {
        assert!(block_index < fs::BLOCK_NUM);
        assert_eq!(buf.len(), fs::BLOCK_SIZE);
        let bufu32: &mut [u32] = &mut [0; fs::BLOCK_SIZE / 4];
        let ptr: *const u8 = buf.as_ptr();
        let ptr: *const u32 = ptr as *const u32;
        for i in 0..bufu32.len() {
            bufu32[i] = unsafe { *(ptr.offset((i as usize).try_into().unwrap())) };
        }
        fs::HDB.write(block_index as u64, 1, bufu32)
    }
    pub fn clear_block(block_index: usize) -> Result<(), ()> {
        assert!(block_index < fs::BLOCK_NUM);
        let buf: &mut [u32] = &mut [0; fs::BLOCK_SIZE / 4];
        fs::HDB.write(block_index as u64, 1, buf)
    }

    pub fn init() -> Result<Vec<fs::file::File>, ()> {
        let files = BlockDevice::get_files();
        //前128个块中包含全部文件信息。
        //这其中最后一个区块用于空闲区管理
        return files;
    }

    pub fn get_files() -> Result<Vec<fs::file::File>, ()> {
        let mut files = Vec::new();
        //前256个块中包含全部文件信息。
        let super_blocks_num = fs::SUPER_BLOCKS_NUM;
        //这其中最后一个区块用于空闲区管理
        for i in 0..super_blocks_num {
            match BlockDevice::read_block(i) {
                Err(_) => panic!("read block err"),
                Ok(v) => {
                    //每个block存放16个文件的元信息。
                    for j in 0..META_INFO_PER_BLOCK {
                        let file_meta_info: &[u8] = &v[j * META_INFO_LEN
                            ..j * META_INFO_LEN + (fs::BLOCK_SIZE / META_INFO_PER_BLOCK)];
                        let file = match BlockDevice::parse_file_meta_info(file_meta_info) {
                            Some(file) => file,
                            None => continue,
                        };
                        files.push(file);
                    }
                }
            }
        }
        return Ok(files);
    }

    fn parse_file_meta_info(meta: &[u8]) -> Option<fs::file::File> {
        //meta的长度应为 BLOCK_SIZE / 16
        assert_eq!(meta.len(), fs::BLOCK_SIZE / 16);
        let block_index: usize;
        let block_count: usize;
        let u8_length;
        //前16位为块起始索引
        //前17-32位为块的数量
        //前33-48位为字节数量
        block_index = ((meta[0] as usize) << 8 | meta[1] as usize) as usize;
        block_count = ((meta[2] as usize) << 8 | meta[3] as usize) as usize;
        u8_length = ((meta[4] as usize) << 8 | meta[5] as usize) as usize;
        if block_index <= fs::SUPER_BLOCKS_NUM {
            return None;
        }
        let mut name = String::new();
        for c in 6..meta.len() {
            if meta[c] == (0 as u8) {
                break;
            }
            name.push(meta[c] as char);
        }
        Some(fs::file::File {
            block_index,
            name,
            block_count,
            u8_length,
        })
    }

    pub fn get_unused_blocks() -> Result<Vec<UnusedBlocks>, ()> {
        return UnusedBlocks::parse_bit_map_block();
    }

    pub fn insert_file(name: String, content: &String) -> Result<fs::File, ()> {
        let block_count = content.len() / fs::BLOCK_SIZE + 1;
        let u8_length = content.len();
        match UnusedBlocks::get_usable_blocks_for_size(block_count) {
            Some(unused_block) => {
                let block_index = unused_block.block_index;
                let f = fs::File {
                    block_index,
                    name,
                    block_count,
                    u8_length,
                };
                match BlockDevice::save_file_data(&f, content) {
                    Ok(_) => {
                        UnusedBlocks::use_blocks(block_index, block_count).unwrap();
                    }
                    Err(_) => return Err(()),
                }

                Ok(f)
            }
            None => Err(()),
        }
    }

    pub fn check_file_name_exist(f: &fs::File) -> bool {
        let bucket_num = BlockDevice::hash_file_name(&f.name);
        let block = BlockDevice::read_block(bucket_num).unwrap();
        for j in 0..META_INFO_PER_BLOCK {
            let file_meta_info: &[u8] = &block
                [j * META_INFO_LEN..j * META_INFO_LEN + (fs::BLOCK_SIZE / META_INFO_PER_BLOCK)];
            match BlockDevice::parse_file_meta_info(file_meta_info) {
                Some(file) => {
                    if file.name == f.name {
                        return true;
                    }
                }
                None => continue,
            };
        }
        return false;
    }

    pub fn save_file_data(f: &fs::File, content: &String) -> Result<(), ()> {
        if BlockDevice::check_file_name_exist(&f) {
            return Err(());
        }
        //1. save content first
        let content = content.as_bytes();

        for i in 0..f.block_count {
            //kprintln!("file content, {:?}", content);
            if i < f.block_count - 1 {
                match BlockDevice::write_block(
                    f.block_index + i,
                    &content[i * fs::BLOCK_SIZE..(i + 1) * fs::BLOCK_SIZE],
                ) {
                    Ok(_) => continue,
                    Err(_) => return Err(()),
                }
            } else {
                let bufori = &content[i * fs::BLOCK_SIZE..];
                let mut buf = [0; fs::BLOCK_SIZE];
                for i in 0..bufori.len() {
                    buf[i] = bufori[i];
                }
                // kprint!("content buf:");
                // for i in 0..buf.len(){
                //     kprint!("{},", buf[i]);
                // }
                match BlockDevice::write_block(f.block_index + i, &mut buf) {
                    Ok(_) => continue,
                    Err(_) => return Err(()),
                }
            }
        }
        //2. save meta data later
        let mut meta = [0; META_INFO_LEN];
        let mut l = (f.block_index >> 8) as u8;
        let mut r = f.block_index as u8;
        meta[0] = l;
        meta[1] = r;
        l = (f.block_count >> 8) as u8;
        r = f.block_count as u8;
        meta[2] = l;
        meta[3] = r;
        l = (f.u8_length >> 8) as u8;
        r = f.u8_length as u8;
        meta[4] = l;
        meta[5] = r;
        let mut name_bytes = f.name.clone().into_bytes();
        let name_array = &mut name_bytes[..];
        for i in 0..name_array.len() {
            meta[6 + i] = name_array[i];
        }
        let bucket = BlockDevice::hash_file_name(&f.name);
        let mut block = BlockDevice::read_block(bucket).unwrap();

        for i in 0..META_INFO_PER_BLOCK {
            let file_meta_info: &mut [u8] = &mut block
                [i * META_INFO_LEN..i * META_INFO_LEN + (fs::BLOCK_SIZE / META_INFO_PER_BLOCK)];
            if file_meta_info[0] == 0 && file_meta_info[1] == 0 {
                //有空位
                // for j in 0..META_INFO_LEN{
                //     block[i*META_INFO_LEN + j] = file_meta_info[j];
                // }
                for j in 0..META_INFO_LEN {
                    file_meta_info[j] = meta[j];
                }
                break;
            }
        }
        //kprintln!("meta block after write: {:?}", block);
        BlockDevice::write_block(bucket, &mut block[..])
    }

    pub fn hash_file_name(name: &String) -> usize {
        let mut sum = 0;
        for c in name.chars() {
            sum += c as usize;
        }
        sum % fs::SUPER_BLOCKS_NUM
    }

    pub fn get_file_meta_data_base_on_name(name: &String) -> Result<fs::File, ()> {
        let bucket = BlockDevice::hash_file_name(&name);
        let block = BlockDevice::read_block(bucket).unwrap();
        for j in 0..META_INFO_PER_BLOCK {
            let file_meta_info: &[u8] =
                &block[j * META_INFO_LEN..j * META_INFO_LEN + (fs::BLOCK_SIZE / META_INFO_PER_BLOCK)];
            let file = match BlockDevice::parse_file_meta_info(file_meta_info) {
                Some(file) => {
                    if file.name == name.clone(){
                        return Ok(file);
                    }
                },
                None => continue,
            };
        }
        return Err(());

    }
}

pub fn transfrom_to_u32_array(buf: &[u8], bufu32: &mut [u32]) {
    assert_eq!(buf.len(), bufu32.len() * 4);

    let ptr: *const u8 = buf.as_ptr();
    let ptr: *const u32 = ptr as *const u32;
    let bufu32ptr = bufu32.as_mut_ptr();
    unsafe { *bufu32ptr = *ptr }
}

pub fn transfrom_to_u8_array(buf: &mut [u8], bufu32: &[u32]) {
    assert_eq!(buf.len(), bufu32.len() * 4);

    let ptr: *const u32 = bufu32.as_ptr();
    let ptr: *const u8 = ptr as *const u8;
    let bufu8ptr = buf.as_mut_ptr();
    unsafe { *bufu8ptr = *ptr }
}

#[derive(core::fmt::Debug)]
pub struct UnusedBlocks {
    pub block_index: usize,
    pub count: usize,
}

impl UnusedBlocks {
    pub fn new(block_index: usize, count: usize) -> UnusedBlocks {
        UnusedBlocks { block_index, count }
    }

    pub fn parse_bit_map_block() -> Result<Vec<UnusedBlocks>, ()> {
        match BlockDevice::read_block(fs::UNUSED_BLOCKS_BITMAP_INDEX) {
            Ok(v) => {
                //kprintln!("prepare parse bit map, {:?}", v);
                let mut res = Vec::new();
                let bmap = BitMap::new(v);
                let mut count = 0;
                let mut block_index = 0;
                for i in 0..fs::BLOCK_NUM {
                    if bmap.check_exists((i as usize).try_into().unwrap()) {
                        if count != 0 {
                            res.push(UnusedBlocks { block_index, count });
                            count = 0;
                        }
                    } else {
                        if count == 0 {
                            block_index = i;
                        }
                        count += 1;
                    }
                }
                if count != 0 {
                    res.push(UnusedBlocks { block_index, count });
                }
                return Ok(res);
            }
            Err(_) => Err(()),
        }
    }

    pub fn get_usable_blocks_for_size(block_count: usize) -> Option<UnusedBlocks> {
        match UnusedBlocks::parse_bit_map_block() {
            Ok(unused) => {
                for u in unused {
                    if u.count > block_count {
                        return Some(u);
                    }
                }
                return None;
            }
            Err(_) => None,
        }
    }

    pub fn use_blocks(block_index: usize, count: usize) -> Result<(), ()> {
        match UnusedBlocks::get_unused_bitmap_block() {
            Ok(b) => {
                let mut bmap = BitMap::new(b);
                for i in 0..count {
                    let i: usize = (i as usize).try_into().unwrap();
                    bmap.set_2_exist(block_index + i);
                }
                BlockDevice::write_block(fs::UNUSED_BLOCKS_BITMAP_INDEX, &mut bmap.v)
            }
            Err(_) => Err(()),
        }
    }

    pub fn get_unused_bitmap_block() -> Result<Vec<u8>, ()> {
        BlockDevice::read_block(fs::UNUSED_BLOCKS_BITMAP_INDEX)
    }
}

pub struct BitMap {
    pub v: Vec<u8>,
}

impl BitMap {
    pub fn new(v: Vec<u8>) -> BitMap {
        BitMap { v }
    }

    pub fn check_exists(&self, index: usize) -> bool {
        assert!(index < self.v.len() * 8);
        let u = self.v[index / 8];
        let offset = 7 - index % 8;
        let flag = (1 as u8) << offset;
        //kprintln!("flag: {}", flag);
        flag & u == flag
    }

    pub fn set_2_exist(&mut self, index: usize) {
        assert!(index < self.v.len() * 8);
        let offset = 7 - index % 8;
        let flag = (1 as u8) << offset;
        self.v[index / 8] = flag | self.v[index / 8];
    }

    pub fn set_2_not_exist(&mut self, index: usize) {
        assert!(index < self.v.len() * 8);
        let offset = 7 - index % 8;
        self.v[index / 8] &= !((1 as u8) << offset);
    }
}
