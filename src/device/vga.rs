// #![allow(dead_code)]

// use crate::sync::IrqLock;
// use core::fmt;
// use core::ptr::Unique;
// use volatile::Volatile;

// const MAX_HEIGHT: usize = 100;
// static origin_height: usize = 0;

// const SCREEN_HEIGHT: usize = 25;
// const SCREEN_WIDTH: usize = 80;

// pub static VGA: IrqLock<Writer> = IrqLock::new(Writer {
//     row_position: 0,
//     column_position: 0,
//     color_code: ColorCode::new(Color::LightGray, Color::Black),
//     buffer: unsafe { Unique::new_unchecked(0xb8000 as *mut _) },
// });

// pub struct Writer {
//     row_position: usize,
//     column_position: usize,
//     color_code: ColorCode,
//     buffer: Unique<Buffer>,
// }

// impl Writer {
//     pub fn clear_screen(&mut self) {
//         for row in 0..SCREEN_HEIGHT {
//             self.clear_row(row);
//         }
//     }

//     fn write_byte(&mut self, byte: u8) {
//         match byte {
//             b'\n' => self.new_line(),
//             0x8 => { //退格键ascii
//                 let row;
//                 let col;
//                 if self.column_position == 0{
//                     self.row_position -= 1;
//                     self.column_position = SCREEN_WIDTH - 1;
//                     row = self.row_position;
//                     col = self.column_position;
//                 }else{
//                     row = self.row_position;
//                     col = self.column_position - 1;
//                 }
//                 let color_code = self.color_code;
//                 self.buffer().chars[row][col].write(ScreenChar {
//                     ascii_character: b' ',
//                     color_code: color_code,
//                 });
//                 self.column_position -= 1;
                
//             }
//             byte => {
//                 if self.column_position >= SCREEN_WIDTH {
//                     self.new_line();
//                 }

//                 let row = self.row_position;
//                 let col = self.column_position;

//                 let color_code = self.color_code;
//                 self.buffer().chars[row][col].write(ScreenChar {
//                     ascii_character: byte,
//                     color_code: color_code,
//                 });
//                 self.column_position += 1;
//             }
//         }
//     }

//     fn write_string(&mut self, s: &str) {
//         for byte in s.bytes() {
//             match byte {
//                 // printable ASCII byte or newline
//                 0x20..=0x7e | b'\n' | 0x8 => self.write_byte(byte),
//                 // not part of printable ASCII range
//                 _ => self.write_byte(0xfe),
//             }
//         }
//     }

//     fn buffer(&mut self) -> &mut Buffer {
//         unsafe { self.buffer.as_mut() }
//     }

//     fn new_line(&mut self) {
//         if self.row_position == SCREEN_HEIGHT - 1{
//             for row in 1..SCREEN_HEIGHT {
//                 for col in 0..SCREEN_WIDTH {
//                     let buffer = self.buffer();
//                     let character = buffer.chars[row][col].read();
//                     buffer.chars[row - 1][col].write(character);
//                 }
//             }
//             self.clear_row(SCREEN_HEIGHT - 1);
//             self.row_position = SCREEN_HEIGHT - 1;
//             self.column_position = 0;
//         }else{
//             self.row_position += 1;
//             self.column_position = 0;
//         }
//     }

//     fn clear_row(&mut self, row: usize) {
//         let blank = ScreenChar {
//             ascii_character: b' ',
//             color_code: self.color_code,
//         };

//         for col in 0..SCREEN_WIDTH {
//             self.buffer().chars[row][col].write(blank);
//         }
//     }
// }

// impl fmt::Write for Writer {
//     fn write_str(&mut self, s: &str) -> ::core::fmt::Result {
//         self.write_string(s);
//         Ok(())
//     }
// }

// struct Buffer {
//     pub chars: [[Volatile<ScreenChar>; SCREEN_WIDTH]; SCREEN_HEIGHT],
// }

// #[allow(dead_code)]
// #[derive(Debug, Clone, Copy)]
// #[repr(u8)]
// pub enum Color {
//     Black = 0,
//     Blue = 1,
//     Green = 2,
//     Cyan = 3,
//     Red = 4,
//     Magenta = 5,
//     Brown = 6,
//     LightGray = 7,
//     DarkGray = 8,
//     LightBlue = 9,
//     LightGreen = 10,
//     LightCyan = 11,
//     LightRed = 12,
//     Pink = 13,
//     Yellow = 14,
//     White = 15,
// }

// #[derive(Debug, Clone, Copy)]
// pub struct ColorCode(u8);

// impl ColorCode {
//     pub const fn new(foreground: Color, background: Color) -> ColorCode {
//         ColorCode((background as u8) << 4 | (foreground as u8))
//     }
// }

// #[derive(Debug, Clone, Copy)]
// #[repr(C)]
// pub struct ScreenChar {
//     pub ascii_character: u8,
//     pub color_code: ColorCode,
// }







#![allow(dead_code)]

use crate::sync::IrqLock;
use core::fmt;
use core::ptr::Unique;
use volatile::Volatile;

const MAX_HEIGHT: usize = 100;

const SCREEN_HEIGHT: usize = 25;
const SCREEN_WIDTH: usize = 80;

pub static VGA: IrqLock<Writer> = IrqLock::new(Writer {
    row_position: 0,
    column_position: 0,
    color_code: ColorCode::new(Color::LightGray, Color::Black),
    buffer: unsafe { Unique::new_unchecked(0xb8000 as *mut _) },
    display_buffer: DisplayBuffer::new()
});

pub struct Writer {
    row_position: usize,
    column_position: usize,
    color_code: ColorCode,
    buffer: Unique<Buffer>,
    display_buffer: DisplayBuffer
}

impl Writer {
    pub fn clear_screen(&mut self) {
        for row in 0..MAX_HEIGHT {
            self.clear_row(row);
        }
        self.display_buffer.origin = 0;
        self.refresh();
    }

    fn write_byte(&mut self, byte: u8) {

        if self.row_position - self.display_buffer.origin != SCREEN_HEIGHT - 1 && self.display_buffer.origin != 0{
            if self.row_position < SCREEN_HEIGHT + 1{
                self.display_buffer.origin = 0;
            }else{
                self.display_buffer.origin = self.row_position - SCREEN_HEIGHT + 1;
            }
            self.refresh();
        }

        match byte {
            b'\n' => self.new_line(),
            0x8 => { //退格键ascii
                let row;
                let col;
                let origin = self.display_buffer.origin;
                let mut refresh = false;
                // kprint!("ori {}, row pos {}", origin, self.row_position);
                // return;
                if self.row_position - origin != SCREEN_HEIGHT - 1 && origin != 0 {
                    refresh = true;
                    self.display_buffer.origin = self.row_position - SCREEN_HEIGHT;
                }   

                if self.column_position == 0{
                    self.row_position -= 1;
                    self.column_position = SCREEN_WIDTH - 1;
                    row = self.row_position;
                    col = self.column_position;
                    if origin != 0{
                        refresh = true;
                        self.display_buffer.origin -= 1;
                    }
                }else{
                    row = self.row_position;
                    col = self.column_position - 1;
                    self.column_position -= 1;
                }
                let color_code = self.color_code;
                let screen_char = ScreenChar {
                    ascii_character: b' ',
                    color_code: color_code,
                };
                self.buffer().chars[(row as u64 - origin as u64) as usize][col].write(screen_char);
                self.display_buffer.chars[row][col] = screen_char;

                if refresh {
                    self.refresh();
                }
                
            }
            byte => {
                if self.column_position >= SCREEN_WIDTH {
                    self.new_line();
                }


                let row = self.row_position;
                let col = self.column_position;

                let color_code = self.color_code;
                let screen_char = ScreenChar {
                    ascii_character: byte,
                    color_code: color_code,
                };
                let origin = self.display_buffer.origin;
                self.buffer().chars[(row as u64 - origin as u64) as usize][col].write(screen_char);
                self.display_buffer.chars[row][col] = screen_char;
                self.column_position += 1;
            }
        }
    }

    fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // printable ASCII byte or newline
                0x20..=0x7e | b'\n' | 0x8 => self.write_byte(byte),
                0x4 => self.scroll(true), //up
                0x5 => self.scroll(false), //down
                // not part of printable ASCII range
                _ => self.write_byte(0xfe),
            }
        }
    }

    fn buffer(&mut self) -> &mut Buffer {
        unsafe { self.buffer.as_mut() }
    }

    fn new_line(&mut self) {
        //TODO
        if self.row_position == MAX_HEIGHT - 1{
            for row in 1..MAX_HEIGHT {
                for col in 0..SCREEN_WIDTH {
                    let buffer = &mut self.display_buffer;
                    let character = buffer.chars[row][col];
                    buffer.chars[row - 1][col] = character;
                }
            }
            self.clear_row(MAX_HEIGHT - 1);
            self.column_position = 0;
            self.display_buffer.origin = MAX_HEIGHT - SCREEN_HEIGHT;
        }else{
            self.row_position += 1;
            self.column_position = 0;
            if self.row_position >= SCREEN_HEIGHT{
                self.display_buffer.origin += 1;
            }
        }
        self.refresh();
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };

        for col in 0..SCREEN_WIDTH {
            self.display_buffer.chars[row][col] = blank;
        }
    }

    fn refresh(&mut self){
        assert!(self.display_buffer.origin <= (MAX_HEIGHT - SCREEN_HEIGHT));
        //adjust
        for row in 0..SCREEN_HEIGHT{
            for col in 0..SCREEN_WIDTH{
                let origin = self.display_buffer.origin;
                let screen_char = self.display_buffer.chars[row + origin][col];
                self.buffer().chars[row][col].write(screen_char);
            }
        }
    }

    fn scroll(&mut self, up: bool){
        if up && self.display_buffer.origin > 0{
            self.display_buffer.origin -= 1;
            self.refresh();
        }else if !up && self.display_buffer.origin < (MAX_HEIGHT - SCREEN_HEIGHT){
            self.display_buffer.origin += 1;
            self.refresh();
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> ::core::fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

struct Buffer {
    pub chars: [[Volatile<ScreenChar>; SCREEN_WIDTH]; SCREEN_HEIGHT],
}

struct DisplayBuffer {
    pub chars: [[ScreenChar; SCREEN_WIDTH]; MAX_HEIGHT],
    origin: usize
}

impl DisplayBuffer{
    pub const fn new() -> DisplayBuffer{
        DisplayBuffer {
            chars: [[ScreenChar{
                ascii_character: b' ',
                color_code: ColorCode::new(Color::Black, Color::Black)
            }; SCREEN_WIDTH]; MAX_HEIGHT],
            origin: 0
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[derive(Debug, Clone, Copy)]
pub struct ColorCode(u8);

impl ColorCode {
    pub const fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct ScreenChar {
    pub ascii_character: u8,
    pub color_code: ColorCode,
}
