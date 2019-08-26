#[cfg(target_arch = "x86_64")]
#[macro_use]
pub mod x86_64;

#[cfg(target_arch = "x86_64")]
pub use self::x86_64::*;

use spin::Mutex;

pub static mut tick: Mutex<u64> = Mutex::new(0);
