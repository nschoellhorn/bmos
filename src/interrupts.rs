use core::fmt::Write;
use core::sync::atomic::{AtomicU8, Ordering};

use lazy_static::lazy_static;
use pc_keyboard::layouts::Us104Key;
use pc_keyboard::{HandleControl, Keyboard, ScancodeSet1};
use pic8259::ChainedPics;
use spin::Mutex;
use x86_64::instructions::port::Port;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

use bmos_std::io::IOChannel;

use crate::cpu;
use crate::debug;
use crate::events::SystemEvent;
use crate::events::EVENT_QUEUE;
use crate::gdt::{DOUBLE_FAULT_IST_INDEX, TIMER_IST_INDEX};
use crate::keyboard::KeyEvent;
use crate::serial::SERIAL;
use crate::{CONSOLE, SCHEDULER, TERMINAL};
use x86_64::instructions::interrupts::{without_interrupts, are_enabled};

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

const MAX_TICKS: u8 = 5;

pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

static mut TICKS: AtomicU8 = AtomicU8::new(0);

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.invalid_opcode.set_handler_fn(invalid_opcode_handler);
        idt.general_protection_fault.set_handler_fn(protection_fault_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(DOUBLE_FAULT_IST_INDEX);
        }

        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(__timer_interrupt_handler);
        //idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_handler);
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

extern "x86-interrupt" fn page_fault_handler(stack_frame: InterruptStackFrame, error_code: PageFaultErrorCode) {
    panic!("Page Fault: {:?}, Error Code: {:?}", stack_frame, error_code);
}

extern "x86-interrupt" fn protection_fault_handler(stack_frame: InterruptStackFrame, error_code: u64) {
    panic!("General Protection Fault: {:?}, Error Code: {:?}", stack_frame, error_code);
}

extern "x86-interrupt" fn invalid_opcode_handler(stack_frame: InterruptStackFrame) {
    panic!("Invalid Opcode: {:?}", stack_frame);
}

extern "x86-interrupt" {
    fn __timer_interrupt_handler(stack_frame: InterruptStackFrame);
}

#[no_mangle]
extern "C" fn timer_handler(_: InterruptStackFrame) {
    unsafe {
        if TICKS.load(Ordering::SeqCst) == MAX_TICKS {
            TICKS.store(0, Ordering::SeqCst);
            let next = SCHEDULER.as_mut().unwrap().pick_next();

            PICS.lock()
                .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());

            if let Some((current, next)) = next {
                cpu::switch_context(&*current, &*next);
            }
        } else {
            let _ = TICKS.fetch_add(1, Ordering::SeqCst);
            PICS.lock()
                .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
        }
    }
}

global_asm!(include_str!("asm/interrupt.s"), options(att_syntax));

extern "x86-interrupt" fn syscall_handler(_: InterruptStackFrame) {
    let syscall_number = cpu::read_rax();
    match syscall_number {
        1 => {
            // print()
            let data_start = cpu::read_rdi() as *const u8;
            let length = cpu::read_rsi();
            let io_channel = IOChannel::from_u32(cpu::read_rdx() as u32).unwrap();
            /*debug!(
                "Arguments: data_start = {:#?}, length = {}, io_channel = {:?}",
                data_start, length, io_channel
            );*/

            let data_slice = unsafe { core::slice::from_raw_parts(data_start, length as usize) };
            let string = core::str::from_utf8(data_slice);

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

extern "x86-interrupt" fn keyboard_handler(_: InterruptStackFrame) {
    without_interrupts(|| {
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
            EVENT_QUEUE
                .lock()
                .push_back(SystemEvent::KeyboardEvent(event));

            PICS.lock()
                .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
        }
    });
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
