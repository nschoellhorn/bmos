#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(asm)]
#![feature(naked_functions)]
#![feature(global_asm)]
#![feature(default_alloc_error_handler)]

extern crate alloc;

use crate::console::Console;
use crate::keyboard::KEYBOARD_REGISTRY;
use crate::scheduler::RoundRobinScheduler;
use crate::terminal::Terminal;
use alloc::boxed::Box;
use bmos_shell::BmShell;
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use graphics::{Framebuffer, GraphicsSettings};
use psf::Font;
use spin::Mutex;

mod console;
mod cpu;
mod gdt;
mod graphics;
mod interrupts;
mod keyboard;
mod memory;
mod scheduler;
mod serial;
mod terminal;
mod threading;

const FONT: &'static [u8] = include_bytes!("../font.psf");

entry_point!(kernel_main);

static mut GRAPHICS_SETTINGS: Option<GraphicsSettings> = None;
static mut FRAMEBUFFER: Option<Mutex<Framebuffer>> = None;
static mut BASE_FONT: Option<Font> = None;
pub static mut CONSOLE: Option<Console<'static>> = None;
pub static mut TERMINAL: Option<Terminal<'static>> = None;
pub static mut SCHEDULER: Option<RoundRobinScheduler> = None;

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    gdt::init();
    memory::init(
        &boot_info.memory_regions,
        boot_info.physical_memory_offset.into_option().unwrap(),
    );

    unsafe {
        let initial_task = threading::build("main", || {
            debug!("Hello from main thread");
        });
        SCHEDULER = Some(RoundRobinScheduler::new(initial_task));
    }

    interrupts::init();
    keyboard::init();

    threading::spawn("test", || {
        debug!("Printing from a nice thread!");
    });

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

        registry.register(TERMINAL.as_mut().unwrap());

        let shell = Box::new(BmShell::new());
        TERMINAL.as_mut().unwrap().launch_shell(shell);
    };

    loop {
        x86_64::instructions::hlt();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    debug!("{}", info);
    loop {}
}
