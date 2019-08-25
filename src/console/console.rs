use alloc::vec::Vec;
use crate::device;
use alloc::string::String;
use core::iter::FromIterator;

pub extern "C" fn console_process(){
    //device::console::clear_screen();
    kprintln!("In main process!\n");
    init();
    //serial_println!("In main process!\n");
    loop{
        #[cfg(feature = "vga")]
        {
            use device::keyboard::ps2 as kbd;
            let mut line = Vec::new();
            kprint!("==> ");
            loop{
                match kbd::read_c() {
                    Some(c) => {
                        kprint!("{}", c);
                        if c == '\n'{
                            break;
                        }
                        if c == 'Q'{
                            line.pop();
                        }
                        line.push(c);
                    },
                    None => continue
                }
            }
            let command = String::from_iter(line.into_iter());
            unsafe{
                match commands.getCommand(command){
                    Some(c) => {
                        let handler = &c.handler;
                        handler();
                    },
                    None => kprintln!("no command found!")
                }
            }
        }
    }
}

pub struct Command{
    pub name: String,
    pub handler: fn() -> ()
}

impl Command{
    pub fn new(name: String, f: fn()->()) -> Command
    {
        Command{
            name,
            handler: f
        }
    }
}

pub struct Commands{
    pub commands: Vec<Command>
}

impl Commands{
    pub const fn new() -> Commands{
        Commands{
            commands: Vec::new(),
        }
    }
    pub fn add(&mut self, name: String, f: fn()->()){
        let command = Command::new(name.clone(), f);
        self.commands.push(command);
    }
    pub fn getCommand(&self, name: String) -> Option<&Command>{
        //kprintln!("com = {}", name);
        for c in self.commands.iter(){
            //kprintln!("inside:comname: {}", c.name);
            if c.name == name{
                return Some(c);
            }
        }
        return None
    }
}

static mut commands: Commands = Commands::new();

pub fn init(){
    //add Commands
    unsafe {
        commands.add(String::from("hey!"), ||{
            kprintln!(":)");
        });
        commands.add(String::from("fsj"), ||{
            kprintln!("my love !!!!!");
        })
    }
}