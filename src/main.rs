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
mod gdt;

use crate::graphics::psf2_t;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.double_fault.set_handler_fn(double_fault_handler);
        idt
    };
}

const FONT: &'static [u8] = include_bytes!("../font.psf");

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    gdt::init();
    debug!("test");
    IDT.load();

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

    let font = match psf::Font::parse_font_data(FONT) {
        Err(error) => panic!("Failed to parse font data: {:?}", error),
        Ok(font) => {
            debug!("Parsed PSF font: {:?}", &font);
            font
        }
    };

    {
        let mut framebuffer = framebuffer.lock();
        framebuffer.clear();
        framebuffer.draw_rect(100, 100, 100, 100, 0x6441A4);
    }

    let mut console = console::Console::init(&graphics_settings, &framebuffer, &font);
    console.println("Hello");
    console.println("World");

    loop {
        //println!("Tick");
        unsafe { asm!("hlt") };
    }
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) -> ! {
    panic!(
        "EXCEPTION: DOUBLE FAULT, Error Code: {}\n{:#?}",
        error_code, stack_frame
    );
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    debug!("{}", info);
    loop {}
}
