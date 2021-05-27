use core::fmt;

use lazy_static::lazy_static;
use spin::Mutex;
use uart_16550::SerialPort;

lazy_static! {
    pub static ref SERIAL: Mutex<SerialLine> = {
        let mut serial = unsafe { SerialPort::new(0x3f8) };
        serial.init();
        Mutex::new(SerialLine::new(serial))
    };
}

/// Like the `print!` macro in the standard library, but prints to the VGA text buffer.
#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => ($crate::serial::_debug(format_args!($($arg)*)));
}

pub struct SerialLine {
    serial_port: SerialPort,
}

impl SerialLine {
    pub fn new(serial_port: SerialPort) -> Self {
        Self { serial_port }
    }
}

impl fmt::Write for SerialLine {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        s.chars().for_each(|c| self.serial_port.send(c as u8));
        Ok(())
    }
}

#[doc(hidden)]
pub fn _debug(args: fmt::Arguments) {
    use core::fmt::Write;
    let mut serial = SERIAL.lock();
    serial.write_fmt(args).unwrap();
    serial.write_str("\n").unwrap();
}
