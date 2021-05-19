use crate::threading::Thread;

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

global_asm!(include_str!("cpu.s"));

pub fn switch_context(from_thread: &mut Thread, to_thread: &Thread) {
    unsafe {
        __switch_context(
            from_thread as *mut _ as *mut core::ffi::c_void,
            to_thread as *const _ as *const core::ffi::c_void,
        );
    }
}

extern "C" {
    fn __switch_context(from_thread: *mut core::ffi::c_void, to_thread: *const core::ffi::c_void);
}
