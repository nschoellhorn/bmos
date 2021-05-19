use crate::debug;
use crate::memory;
use crate::SCHEDULER;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::string::ToString;
use core::ffi::c_void;
use x86_64::VirtAddr;

/// Each thread has its own stack. The stack is exactly one page of memory, which equals 4KiB.
#[repr(C)]
pub struct Thread {
    /// The stack pointer represents the thread's stack state before switching to another one.
    /// It is the first field so we can re-use the pointer to this struct as a pointer to the stack pointer.
    pub stack_pointer: VirtAddr,
    pub name: String,
}

impl Eq for Thread {}
impl PartialEq for Thread {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

unsafe fn initialize_stack(base_addr: VirtAddr, thread: &Thread, runnable: *const c_void) {
    let base_ptr = base_addr.as_mut_ptr::<u64>();
    let slice = unsafe { core::slice::from_raw_parts_mut(base_ptr, 4096 / 8) };
    slice.fill(0);
    unsafe {
        slice[slice.len() - 4] = runnable as u64;
        slice[slice.len() - 3] =
            thread_start as *const extern "C" fn(*mut c_void) -> *mut c_void as u64;
        slice[slice.len() - 2] = __cleanup_thread as *const extern "C" fn() -> () as u64;
        slice[slice.len() - 1] = thread as *const Thread as u64;
    }

    extern "C" fn thread_start(main: *mut c_void) -> *mut c_void {
        unsafe {
            Box::from_raw(main as *mut Box<dyn FnOnce()>)();
        }
        core::ptr::null_mut()
    }
}

pub(crate) fn build<F>(name: &str, f: F) -> Thread
where
    F: FnOnce() -> (),
    F: Send + 'static,
{
    let stack_page = memory::allocate_kernel_page().expect("Failed to allocate kernel memory");
    let stack_addr = stack_page.start_address() + stack_page.size();

    // Weird hack to get the raw pointer to the closure/function
    let boxed_closure: Box<dyn FnOnce()> = Box::new(f);
    let pointer = Box::into_raw(Box::new(boxed_closure));

    let thread = Thread {
        name: name.to_string(),
        stack_pointer: stack_addr - (9 * 8) as u64,
    };

    unsafe {
        initialize_stack(
            stack_page.start_address(),
            &thread,
            pointer as *const c_void,
        );
    }

    thread
}

pub(crate) fn spawn<F>(name: &str, f: F)
where
    F: FnOnce() -> (),
    F: Send + 'static,
{
    let task = build(name, f);

    unsafe {
        SCHEDULER.as_mut().unwrap().add_task(task);
    }
}

extern "C" {
    fn __cleanup_thread();
}

#[no_mangle]
pub unsafe extern "C" fn cleanup_thread(current_thread: *mut Thread) {
    debug!("Cleaning up thread {}", (*current_thread).name);
}

global_asm!(include_str!("threads.s"));
