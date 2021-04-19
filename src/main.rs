#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(asm)]

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use lazy_static::lazy_static;
use spin::Mutex;
use uart_16550::SerialPort;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

mod console;
mod graphics;
mod serial;
#[macro_use]
mod vga_buffer;

use crate::graphics::psf2_t;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.double_fault.set_handler_fn(double_fault_handler);
        idt
    };
}

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    debug!("test");
    IDT.load();
    debug!(
        "interrupts: {}",
        x86_64::instructions::interrupts::are_enabled()
    );
    // First, set up basic graphics and a console to make sure we can print debug stuff
    if let bootloader::boot_info::Optional::None = boot_info.framebuffer {
        panic!("No framebuffer found! This is a problem.");
    }
    let boot_fb = boot_info.framebuffer.as_mut().unwrap();
    let framebuffer = Mutex::new(graphics::Framebuffer::from_boot_info_framebuffer(boot_fb));
    let graphics_settings = graphics::GraphicsSettings {
        width: boot_fb.info().horizontal_resolution as u32,
        height: boot_fb.info().vertical_resolution as u32,
    };

    loop {
        //println!("Tick");
        unsafe { asm!("hlt") };
    }
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    debug!("{}", info);
    loop {}
}
