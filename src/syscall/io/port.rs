use super::io::Io;
/// Source
/// https://github.com/redox-os/syscall/blob/b5101b25cc8452d4233cc6b4e5b4998a862f8c6c/src/io/pio.rs
use core::marker::PhantomData;

#[derive(Copy, Clone)]
pub struct Port<T> {
    port: u16,
    value: PhantomData<T>,
}

impl<T> Port<T> {
    /// Create a PIO from a given port
    pub const fn new(port: u16) -> Self {
        Port::<T> {
            port: port,
            value: PhantomData,
        }
    }
}

/// Read/Write for byte
impl Io for Port<u8> {
    type Value = u8;

    fn read(&self) -> u8 {
        let value: u8;
        unsafe {
            asm!("in $0, $1" : "={al}"(value) : "{dx}"(self.port) : "memory" : "intel", "volatile");
        }
        value
    }

    fn write(&mut self, value: u8) {
        unsafe {
            asm!("out $1, $0"
                 : : "{al}"(value), "{dx}"(self.port) : "memory" : "intel", "volatile");
        }
    }

    // Write 16-bit array to port
    fn write_arr(&mut self, value: &[u8]) {
        unsafe{
            asm!("rep outsb (%esi), %dx"
            :: "{ecx}"(value.len()), "{dx}"(self.port), "{esi}"(value.as_ptr())
            : "ecx", "edi");
        }
    }

    // Read 8-bit array from port
    fn read_arr(&self, value: &mut [u8]) {
        unsafe{
            asm!("rep insb %dx, (%edi)"
            :: "{ecx}"(value.len()), "{dx}"(self.port), "{edi}"(value.as_ptr())
            : "ecx", "edi" : "volatile");
        }
    }
}

/// Read/Write for word
impl Io for Port<u16> {
    type Value = u16;

    fn read(&self) -> u16 {
        let value: u16;
        unsafe {
            asm!("in $0, $1" : "={ax}"(value) : "{dx}"(self.port) : "memory" : "intel", "volatile");
        }
        value
    }

    fn write(&mut self, value: u16) {
        unsafe {
            asm!("out $1, $0"
                 : : "{ax}"(value), "{dx}"(self.port) : "memory" : "intel", "volatile");
        }
    }

    // Write 16-bit array to port
    fn write_arr(&mut self, value: &[u16]) {
        unsafe{
            asm!("rep outsw (%esi), %dx"
            :: "{ecx}"(value.len()), "{dx}"(self.port), "{esi}"(value.as_ptr())
            : "ecx", "edi");
        }
    }

    // Read 16-bit array from port
    fn read_arr(&self, value: &mut [u16]) {
        unsafe{
            asm!("rep insw %dx, (%edi)"
            :: "{ecx}"(value.len()), "{dx}"(self.port), "{edi}"(value.as_ptr())
            : "ecx", "edi" : "volatile");
        }
    }
}

/// Read/Write for doubleword
impl Io for Port<u32> {
    type Value = u32;

    fn read(&self) -> u32 {
        let value: u32;
        unsafe {
            asm!("in $0, $1"
                 : "={eax}"(value) : "{dx}"(self.port) : "memory" : "intel", "volatile");
        }
        value
    }

    fn write(&mut self, value: u32) {
        unsafe {
            asm!("out $1, $0"
                 : : "{eax}"(value), "{dx}"(self.port) : "memory" : "intel", "volatile");
        }
    }


    // Write 32-bit array to port
    fn write_arr(&mut self, value: &[u32]) {
        unsafe{
            asm!("rep outsl (%esi), %dx"
            :: "{ecx}"(value.len()), "{dx}"(self.port), "{esi}"(value.as_ptr())
            : "ecx", "edi");
        }
    }

    // Read 32-bit array from port
    fn read_arr(&self, value: &mut [u32]) {
        unsafe{
            asm!("rep insl %dx, (%edi)"
            :: "{ecx}"(value.len()), "{dx}"(self.port), "{edi}"(value.as_ptr())
            : "ecx", "edi" : "volatile");
        }
    }

}
