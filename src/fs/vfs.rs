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

    pub fn read_file(name: &String) -> Option<String> {
        //kprintln!("prepare reading file: {}", name);
        match VFS::get_file_handle(&name) {
            Some(f) => {
                //kprintln!("file meta data get, {:?}", f);
                match f.read_content() {
                    Ok(s) => Some(s),
                    _ => None,
                }
            }
            None => None,
        }
    }

    pub fn get_file_handle(name: &String) -> Option<fs::File> {
        match fs::block_device::BlockDevice::get_file_meta_data_base_on_name(&name) {
            Ok(f) => {
                return Some(f);
            },
            Err(_) => None
        }
    }

    pub fn create_file(name: &String, content: &String) -> Result<fs::File, ()> {
        fs::block_device::BlockDevice::insert_file(&name, &content)
    }

    pub fn append_file(name: &String, append_content: &String) -> Result<fs::File, ()> {
        let mut old_content = VFS::read_file(name).unwrap_or(String::from(""));
        match VFS::remove_file(name){
            Err(_) => return Err(()),
            _ => ()
        }
        old_content.push_str(append_content);
        VFS::create_file(name, &old_content)
    }

    pub fn remove_file(name: &String) -> Result<(), ()> {
        fs::block_device::BlockDevice::remove_file(&name)
    }

    pub fn change_file_name(old_file_name: &String, new_file_name: &String) -> Result<(), ()>{
        fs::BlockDevice::change_name(&old_file_name, &new_file_name)
    }
}
