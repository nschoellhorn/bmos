#[derive(Debug, Copy, Clone)]
pub enum IOChannel {
    Stdout = 1,
    Serial = 2,
}

impl IOChannel {
    pub fn from_u32(num: u32) -> Option<IOChannel> {
        match num {
            1 => Some(IOChannel::Stdout),
            2 => Some(IOChannel::Serial),
            _ => None,
        }
    }
}

#[macro_export]
macro_rules! kdebug {
    ($($arg:tt)*) => {{
        let mut string = alloc::fmt::format(format_args!($($arg)*));
        string.push('\n');
        ($crate::syscall::print($crate::io::IOChannel::Serial, string.as_str()));
    }}
}

/*pub fn stdout() -> Stdout {}

pub struct Stdout<'a> {
    console: &Console<'a>,
    terminal: &Terminal<'a>,
}

pub trait Write {
    fn write(&mut self, buffer: &[u8]);
}

impl Stdout {}

impl Write for Stdout {
    pub fn write(&mut self, string: &[u8]) {}
}*/
