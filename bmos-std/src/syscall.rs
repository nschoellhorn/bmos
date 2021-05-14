use crate::io::IOChannel;

macro_rules! syscall {
    ($expression:expr) => {
        asm!(
        "mov rax, {syscall:r}",
        "int 0x80",
        syscall = in(reg) $expression
        );
    }
}

pub fn print(channel: IOChannel, string: &str) {
    let length = string.len() as u64;
    let data_ptr = string.as_ptr() as u64;
    unsafe {
        asm!(
        "mov rdi, {string}",
        "mov rsi, {length}",
        "mov rdx, {channel:r}",
        string = in(reg) data_ptr,
        length = in(reg) length,
        channel = in(reg) channel as i32,
        );
        syscall!(1);
    }
}
