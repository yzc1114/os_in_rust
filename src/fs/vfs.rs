use crate::fs;
use alloc::string::String;
use alloc::vec::Vec;
pub struct VFS;

impl VFS {
    pub fn get_file_names() -> Vec<String> {
        let files = fs::block_device::BlockDevice::get_files().unwrap();
        let mut file_names = Vec::with_capacity(files.len());
        for i in 0..files.len() {
            file_names.push(files[i].name.clone());
        }
        return file_names;
    }

    pub fn read_file(name: String) -> Option<String> {
        //kprintln!("prepare reading file: {}", name);
        match fs::File::get_file(&name) {
            Ok(f) => {
                //kprintln!("file meta data get, {:?}", f);
                match f.read_content() {
                    Ok(s) => Some(s),
                    _ => None,
                }
            }
            Err(_) => None,
        }
    }

    pub fn create_file(name: String, content: String) -> Result<(), ()> {
        match fs::File::create_file(name, &content) {
            Ok(f) => Ok(()),
            Err(_) => Err(()),
        }
    }
}
