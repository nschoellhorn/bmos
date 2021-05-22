use crate::cpu;
use crate::debug;
use crate::keyboard::{KeyEvent, KEYBOARD_REGISTRY};
use crate::serial::SERIAL;
use crate::{CONSOLE, SCHEDULER, TERMINAL};
use bmos_std::io::IOChannel;
use core::fmt::Write;
use lazy_static::lazy_static;
use pc_keyboard::layouts::Us104Key;
use pc_keyboard::{HandleControl, Keyboard, ScancodeSet1};
use pic8259_simple::ChainedPics;
use spin::Mutex;
use x86_64::instructions::port::Port;
use x86_64::structures::idt::PageFaultErrorCode;
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
        idt.page_fault.set_handler_fn(segfault_handler);
        idt.invalid_opcode.set_handler_fn(opcode_handler);

        // Handle timer interrupts
        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_handler);
        idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_handler);
        idt[InterruptIndex::Syscall.as_usize()].set_handler_fn(syscall_handler);

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
    Syscall = 0x80,
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
        SCHEDULER.as_mut().unwrap().tick();
    }
}

extern "x86-interrupt" fn opcode_handler(stack_frame: InterruptStackFrame) {
    panic!("INVALID OPCODE");
}

extern "x86-interrupt" fn segfault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    panic!("SEGFAULT: {:?}, error: {:?}", stack_frame, error_code);
}

extern "x86-interrupt" fn syscall_handler(stack_frame: InterruptStackFrame) {
    let syscall_number = cpu::read_rax();
    match syscall_number {
        1 => {
            // print()
            let data_start = cpu::read_rdi() as *const u8;
            let length = cpu::read_rsi();
            let io_channel = IOChannel::from_u32(cpu::read_rdx() as u32).unwrap();
            debug!(
                "Arguments: data_start = {:#?}, length = {}, io_channel = {:?}",
                data_start, length, io_channel
            );

            let data_slice = unsafe { core::slice::from_raw_parts(data_start, length as usize) };
            let string = core::str::from_utf8(data_slice);
            debug!("String Result: {:?}", string);

            match io_channel {
                IOChannel::Stdout => {
                    let cursor = unsafe { TERMINAL.as_ref().unwrap().cursor_position() };
                    let console = unsafe { CONSOLE.as_ref().unwrap() };

                    console.print(string.unwrap(), cursor.column, cursor.row);
                }
                IOChannel::Serial => {
                    let mut serial = SERIAL.lock();
                    serial.write_str(string.unwrap()).unwrap();
                }
            }
        }
        _ => debug!("INVALID SYSCALL NUMBER"),
    }
    debug!("SYSCALL: {}", syscall_number);
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
