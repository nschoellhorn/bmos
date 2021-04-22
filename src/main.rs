#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(asm)]

use crate::console::Console;
use crate::keyboard::{KeyboardHandler, KEYBOARD_REGISTRY};
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use graphics::{Framebuffer, GraphicsSettings};
use keyboard::KeyEvent;
use pc_keyboard::DecodedKey;
use psf::Font;
use spin::Mutex;

mod console;
mod graphics;
mod serial;
#[macro_use]
mod vga_buffer;
mod gdt;
mod interrupts;
mod keyboard;

const FONT: &'static [u8] = include_bytes!("../font.psf");

entry_point!(kernel_main);

static mut HANDLER: Option<ConsoleOutHandler<'static>> = None;
static mut SERIAL_OUT_HANDLER: Option<SerialOutHandler> = None;
static mut GRAPHICS_SETTINGS: Option<GraphicsSettings> = None;
static mut FRAMEBUFFER: Option<Mutex<Framebuffer>> = None;
static mut BASE_FONT: Option<Font> = None;
static mut CONSOLE: Option<Mutex<Console<'static>>> = None;

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    gdt::init();
    interrupts::init();
    keyboard::init();

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

        CONSOLE = Some(Mutex::new(console::Console::init(
            GRAPHICS_SETTINGS.as_ref().unwrap(),
            FRAMEBUFFER.as_ref().unwrap(),
            BASE_FONT.as_ref().unwrap(),
        )));

        let console_keyboard_handler = ConsoleOutHandler {
            console: CONSOLE.as_ref().unwrap(),
        };
        let serial_keyboard_handler = SerialOutHandler;
        SERIAL_OUT_HANDLER = Some(serial_keyboard_handler);
        HANDLER = Some(console_keyboard_handler);

        let registry = KEYBOARD_REGISTRY.as_mut().unwrap();

        registry.register(HANDLER.as_ref().unwrap());
        registry.register(SERIAL_OUT_HANDLER.as_ref().unwrap());
    };

    loop {
        x86_64::instructions::hlt();
    }
}

struct SerialOutHandler;

impl KeyboardHandler for SerialOutHandler {
    fn handle_key_event(&self, event: KeyEvent) {
        debug!("Key pressed: {:?}", event);
    }
}

struct ConsoleOutHandler<'a> {
    console: &'a Mutex<Console<'a>>,
}

impl<'a> KeyboardHandler for ConsoleOutHandler<'a> {
    fn handle_key_event(&self, event: KeyEvent) {
        if let Some(DecodedKey::Unicode(key)) = event.decoded_key() {
            self.console.lock().put_char(key, 0xff0000);
        }
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    debug!("{}", info);
    loop {}
}
