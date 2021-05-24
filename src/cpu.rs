use x86_64::VirtAddr;
use crate::task::Thread;

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

pub fn read_rdx() -> u64 {
    let rdx_value: u64;
    unsafe {
        asm!("mov {}, rdx", out(reg) rdx_value, options(nomem));
    }

    rdx_value
}

pub fn write_rax(rax_value: u64) {
    unsafe {
        asm!("mov rax, {}", in(reg) rax_value, options(nomem));
    }
}

pub unsafe fn init_switch(init_thread: &Thread) {
    __init_switch(init_thread as *const Thread);
}

pub unsafe fn switch_context(previous_thread: &mut Thread, next_thread: &Thread) {
    __switch_context(previous_thread as *mut Thread, next_thread);
}

global_asm!(include_str!("asm/cpu.s"));

extern "C" {
    fn __switch_context(previous_thread: *mut Thread, next_thread: *const Thread);
    fn __init_switch(init_thread: *const Thread);
}
