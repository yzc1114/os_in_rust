use crate::demo_task;
use crate::device;
use crate::fs::VFS;
use crate::syscall;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use core::iter::FromIterator;

pub extern "C" fn console_process() {
    //device::console::clear_screen();
    kprintln!("In main process!\n");
    init();
    //serial_println!("In main process!\n");
    loop {
        #[cfg(feature = "vga")]
        {
            let line = console_get_new_line();
            let command = String::from_iter(line.into_iter());
            unsafe {
                let args: Vec<&str> = command.split_whitespace().collect();
                if args.len() == 0 {
                    continue;
                }
                match commands.getCommand(args[0]) {
                    Some(c) => {
                        let handler = &c.handler;
                        handler(args);
                    }
                    None => kprintln!("no command found!"),
                }
            }
        }
    }
}

fn console_get_new_line() -> Vec<char> {
    use device::keyboard::ps2 as kbd;
    let mut line = Vec::new();
    kprint!("==> ");
    loop {
        match kbd::read_c() {
            Some(c) => {
                if c == '\n' {
                    kprint!("{}", c);
                    break line;
                }
                if c == 0x8 as char {
                    //退格
                    if line.len() > 0 {
                        line.pop();
                        kprint!("{}", c);
                    }
                    continue;
                }
                kprint!("{}", c);
                line.push(c);
            }
            None => continue,
        }
    }
}

pub struct Command {
    pub name: &'static str,
    pub handler: fn(Vec<&str>) -> (),
}

impl Command {
    pub fn new(name: &'static str, f: fn(Vec<&str>) -> ()) -> Command {
        Command { name, handler: f }
    }
}

pub struct Commands {
    pub commands: Vec<Command>,
}

impl Commands {
    pub const fn new() -> Commands {
        Commands {
            commands: Vec::new(),
        }
    }
    pub fn add(&mut self, name: &'static str, f: fn(Vec<&str>) -> ()) {
        let command = Command::new(name, f);
        self.commands.push(command);
    }
    pub fn getCommand(&self, name: &str) -> Option<&Command> {
        //kprintln!("com = {}", name);
        for c in self.commands.iter() {
            //kprintln!("inside:comname: {}", c.name);
            if c.name == name {
                return Some(c);
            }
        }
        return None;
    }
}

static mut commands: Commands = Commands::new();

pub fn init() {
    //add Commands
    unsafe {
        commands.add("hey!", |_| {
            kprintln!(":)");
        });
        commands.add("ls", |_| {
            kprintln!("{:?}", VFS::get_file_names());
        });
        commands.add("readfile", |args| {
            if args.len() != 2 {
                kprintln!("should have 2 args");
                return;
            }
            kprintln!(
                "{}",
                VFS::read_file(String::from(args[1])).unwrap_or(String::from("read file failed!"))
            );
        });
        commands.add("testmultiprocess", |args| {
            if args.len() != 2 {
                kprintln!("should have 2 args");
                return;
            }
            let num: u32 = match String::from(args[1]).parse() {
                Ok(n) => n,
                Err(_) => {
                    kprintln!("second arg must be integer!");
                    return;
                }
            };
            let mut processes = Vec::new();
            kprintln!("peparing {} test processes!", num);
            for i in 0..num {
                let pid = match syscall::create(
                    String::from(String::from("proc: ") + &i.to_string()),
                    0,
                    demo_task::multi_test_process,
                ) {
                    Ok(pid) => pid,
                    Err(_) => continue,
                };
                processes.push(pid);
            }
            kprintln!("prepare over");
            for i in 0..processes.len() {
                syscall::wait(processes[i]);
            }
        });
        commands.add("get_ticks", |_| {
            kprintln!("{}", syscall::get_ticks());
        });
        commands.add("help", |_| {
            for c in &commands.commands {
                kprintln!("{}", c.name);
            }
        });
        commands.add("createfile", |args| {
            if args.len() == 1 {
                kprintln!("should have 2 args");
                kprintln!("2nd arg should be file name");
                return;
            }
            kprintln!("input content, type return twice to end input");
            use device::keyboard::ps2 as kbd;
            let mut content = Vec::new();
            loop {
                let c = kbd::read_c();
                match c {
                    Some(c) => {
                        if c == 0x8 as char {
                            content.pop();
                        } else if c == '\n' && content.last().unwrap_or(&'\0') == &'\n' {
                            break;
                        } else {
                            content.push(c);
                            kprint!("{}", c);
                        }
                    }
                    None => continue,
                }
            }
            match VFS::create_file(args[1].to_string(), String::from_iter(content.into_iter())) {
                Ok(_) => kprintln!("create success"),
                Err(_) => kprintln!("create fail"),
            }
        })
    }
}
