#![macro_export]
macro_rules! call {
    ($expression:expr) => {
        asm!(
        "mov rax, {syscall}",
        "int 0x80",
        syscall = in(reg) $expression
        );
    }
}

pub fn print(string: &str) {
    let length = string.len() as u64;
    let data_ptr = string.as_ptr() as u64;
    unsafe {
        asm!(
        "mov rdi, {string}",
        "mov rsi, {length}",
        string = in(reg) data_ptr,
        length = in(reg) length
        );
        call!(1);
    }
}
