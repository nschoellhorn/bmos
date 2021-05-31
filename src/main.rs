#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(asm)]
#![feature(default_alloc_error_handler)]
#![feature(box_syntax)]
#![feature(global_asm)]

extern crate alloc;

use alloc::boxed::Box;
use alloc::string::String;
use alloc::sync::Arc;
use core::panic::PanicInfo;

use bootloader::{entry_point, BootInfo};
use psf::Font;
use spin::Mutex;

use bmos_shell::BmShell;
use graphics::{Framebuffer, GraphicsSettings};

use crate::console::Console;
use crate::keyboard::KEYBOARD_REGISTRY;
use crate::scheduler::Scheduler;
use crate::terminal::Terminal;

mod console;
mod cpu;
mod gdt;
mod graphics;
mod interrupts;
mod keyboard;
mod memory;
mod scheduler;
mod serial;
mod task;
mod terminal;

const FONT: &'static [u8] = include_bytes!("../font.psf");

entry_point!(kernel_main);

static mut GRAPHICS_SETTINGS: Option<GraphicsSettings> = None;
static mut FRAMEBUFFER: Option<Mutex<Framebuffer>> = None;
static mut BASE_FONT: Option<Font> = None;
pub static mut CONSOLE: Option<Console<'static>> = None;
pub static mut TERMINAL: Option<Terminal<'static>> = None;

pub static mut SCHEDULER: Option<Scheduler> = None;

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    gdt::init();

    keyboard::init();
    memory::init(
        &boot_info.memory_regions,
        boot_info.physical_memory_offset.into_option().unwrap(),
    );

    // First, set up basic graphics and a console to make sure we can print debug stuff
    if let bootloader::boot_info::Optional::None = boot_info.framebuffer {
        panic!("No framebuffer found! This is a problem.");
    }
    let boot_fb = boot_info.framebuffer.as_mut().unwrap();
    unsafe {
        let mut framebuffer = Framebuffer::from_boot_info_framebuffer(boot_fb);
        framebuffer.clear();
        FRAMEBUFFER = Some(Mutex::new(framebuffer));
        GRAPHICS_SETTINGS = Some(GraphicsSettings {
            width: boot_fb.info().horizontal_resolution as u32,
            height: boot_fb.info().vertical_resolution as u32,
        });

        BASE_FONT = Some(match psf::Font::parse_font_data(FONT) {
            Err(error) => panic!("Failed to parse font data: {:?}", error),
            Ok(font) => {
                debug!("Parsed PSF font.");
                font
            }
        });

        CONSOLE = Some(Console::init(
            GRAPHICS_SETTINGS.as_ref().unwrap(),
            FRAMEBUFFER.as_ref().unwrap(),
            BASE_FONT.as_ref().unwrap(),
        ));

        TERMINAL = Some(Terminal::new(CONSOLE.as_ref().unwrap()));

        let registry = KEYBOARD_REGISTRY.as_mut().unwrap();

        // We can ignore the result safely, since the only possible error is "NoAvailableSlot", and that can't happen this early
        let _ = registry.register(TERMINAL.as_mut().unwrap());

        let shell = Box::new(BmShell::new());
        TERMINAL.as_mut().unwrap().launch_shell(shell);
    };

    // Create idle thread to initialize context switching
    let idle_thread = task::build_thread(String::from("__idle"), || unsafe {
        loop {
            // TODO: Add cleanup stuff here
            asm!("hlt");
        }
    });

    unsafe {
        SCHEDULER = Some(Scheduler::new(Arc::new(idle_thread)));
        interrupts::init();

        SCHEDULER.as_ref().unwrap().init();
    }

    // From here, everything is done in threads. Check the __idle Thread for additional initialization steps.
    loop {
        x86_64::instructions::hlt();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    debug!("{}", info);
    loop {}
}
