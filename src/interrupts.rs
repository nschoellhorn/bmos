use core::iter::Scan;

use crate::{
    debug,
    keyboard::{KeyEvent, KEYBOARD_REGISTRY},
};
use lazy_static::lazy_static;
use pc_keyboard::layouts::Us104Key;
use pc_keyboard::{HandleControl, Keyboard, ScancodeSet1};
use pic8259_simple::ChainedPics;
use spin::Mutex;
use x86_64::instructions::port::Port;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.double_fault.set_handler_fn(double_fault_handler);

        // Handle timer interrupts
        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_handler);
        idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_handler);

        idt
    };

    static ref KEYBOARD_PORT: Mutex<Port<u8>> = Mutex::new(Port::new(0x60));
    static ref KEYBOARD: Mutex<Keyboard<Us104Key, ScancodeSet1>> = Mutex::new(Keyboard::new(Us104Key, ScancodeSet1, HandleControl::Ignore));
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard = PIC_1_OFFSET + 1,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

extern "x86-interrupt" fn timer_handler(stack_frame: InterruptStackFrame) {
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

extern "x86-interrupt" fn keyboard_handler(stack_frame: InterruptStackFrame) {
    // SAFETY: The keyboard can't manipulate our memory.
    let scancode = unsafe { KEYBOARD_PORT.lock().read() };

    let mut keyboard = KEYBOARD.lock();
    let (code, state, key) = match keyboard.add_byte(scancode) {
        Ok(Some(event)) => (event.code, event.state, keyboard.process_keyevent(event)),
        _ => {
            unsafe {
                PICS.lock()
                    .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
            }
            return;
        }
    };

    let event = KeyEvent::new(code, state, key);

    unsafe {
        if let Some(registry) = &KEYBOARD_REGISTRY {
            registry.dispatch_event(event);
        }

        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
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

pub fn init() {
    IDT.load();
    unsafe { PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();
}