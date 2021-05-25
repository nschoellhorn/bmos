use alloc::boxed::Box;
use core::mem::size_of;
use x86_64::VirtAddr;
use alloc::string::String;
use core::ffi::c_void;
use crate::{debug, SCHEDULER};
use core::ptr::write_bytes;

// 4 KiB as the default thread stack size, divided by the size of u64, since that is what we put on the stack
const THREAD_STACK_SIZE: usize = 4096 / size_of::<u64>();
const CALLEE_SAVED_REGISTERS: u64 = 6;
const STACK_FRAME_ELEMENTS: u64 = 3;

#[repr(C)]
pub struct Thread {
    stack_pointer: VirtAddr,
    entry: Box<dyn FnOnce()>,
    pub(crate) stack: Box<[u64; THREAD_STACK_SIZE]>,
    name: String,
}

pub(crate) fn build_thread<F>(name: String, entry: F) -> Thread
    where F: FnOnce() + 'static + Send
{
    pub extern "C" fn thread_entry(thread: *mut Thread) -> ! {
        unsafe {
            debug!("thread_entry(): {}", (*thread).name.as_str());

            Box::from_raw(&mut (*thread).entry as *mut Box<dyn FnOnce()>)();

            debug!("Closure ended");

            // Block the thread until we are ready to clean it up
            loop {
                asm!("hlt");
            }
        }
    }

    let mut stack = box [0u64; THREAD_STACK_SIZE];

    stack[stack.len() - 3] = 0x202u64;
    stack[stack.len() - 2] = thread_entry as *const extern "C" fn(*mut c_void) -> ! as u64;
    stack[stack.len() - 1] = _thread_invalid_return as *const extern "C" fn() -> ! as u64;

    let stack_top = VirtAddr::new(stack.last().unwrap() as *const u64 as u64);

    Thread {
        stack,
        stack_pointer: stack_top - ((CALLEE_SAVED_REGISTERS + STACK_FRAME_ELEMENTS - 1) * 8),
        name,
        entry: Box::new(entry),
    }
}

#[no_mangle]
pub extern "C" fn _thread_invalid_return() -> ! {
    panic!("Thread ended unexpectedly");
}

/*#[no_mangle]
pub unsafe extern "C" fn set_current_kernel_stack() {
    let rsp = SCHEDULER.as_ref().unwrap().current_stack();
    set_kernel_stack(rsp + (THREAD_STACK_SIZE * 8) - 0x10u64);
}*/
