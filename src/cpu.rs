pub fn read_rax() -> u64 {
    let rax_value: u64;
    unsafe {
        asm!("mov {}, rax", out(reg) rax_value, options(nomem));
    }

    rax_value
}

pub fn read_rdi() -> u64 {
    let rdi_value: u64;
    unsafe {
        asm!("mov {}, rdi", out(reg) rdi_value, options(nomem));
    }

    rdi_value
}

pub fn read_rsi() -> u64 {
    let rsi_value: u64;
    unsafe {
        asm!("mov {}, rsi", out(reg) rsi_value, options(nomem));
    }

    rsi_value
}

pub fn write_rax(rax_value: u64) {
    unsafe {
        asm!("mov rax, {}", in(reg) rax_value, options(nomem));
    }
}
