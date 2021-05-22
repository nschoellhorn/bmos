use crate::debug;
use crate::memory;
use crate::SCHEDULER;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::string::ToString;
use core::ffi::c_void;
use core::marker::PhantomPinned;
use core::pin::Pin;
use x86_64::VirtAddr;

/// Each thread has its own stack. The stack is exactly one page of memory, which equals 4KiB.
#[repr(C)]
#[derive(Debug)]
pub struct Thread {
    /// The stack pointer represents the thread's stack state before switching to another one.
    /// It is the first field so we can re-use the pointer to this struct as a pointer to the stack pointer.
    pub stack_pointer: VirtAddr,
    pub entry: *mut c_void,
    pub name: String,
    _marker: PhantomPinned,
}

impl Eq for Thread {}
impl PartialEq for Thread {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Drop for Thread {
    fn drop(&mut self) {
        debug!("Dropping {}", self.name);
    }
}

unsafe fn initialize_stack(base_addr: VirtAddr, thread: &Thread, runnable: *const c_void) {
    let base_ptr = base_addr.as_mut_ptr::<u64>();
    let slice = unsafe { core::slice::from_raw_parts_mut(base_ptr, 4096 / 8) };
    slice.fill(0);
    unsafe {
        slice[slice.len() - 1] =
            thread_start as *const extern "C" fn(*mut Thread) -> *mut c_void as u64;
    }

    extern "C" fn thread_start(thread: *mut Thread) -> *mut c_void {
        unsafe {
            debug!("thread_start(): {}", (*thread).name);

            Box::from_raw((*thread).entry as *mut Box<dyn FnOnce()>)();

            debug!("Closure ended");
            cleanup_thread(thread);
        }

        core::ptr::null_mut()
    }
}

pub(crate) fn build<F>(name: &str, f: F) -> Pin<Box<Thread>>
where
    F: FnOnce() -> (),
    F: Send + 'static,
{
    let stack_page = memory::allocate_kernel_page().expect("Failed to allocate kernel memory");
    let stack_addr = stack_page.start_address() + stack_page.size() - 1u64;

    // Weird hack to get the raw pointer to the closure/function
    let boxed_closure: Box<dyn FnOnce()> = Box::new(f);
    let pointer = Box::into_raw(Box::new(boxed_closure));

    let thread = Thread {
        name: name.to_string(),
        entry: pointer as *mut c_void,
        stack_pointer: stack_addr - (7 * 8) as u64,
        _marker: PhantomPinned,
    };

    debug!(
        "Built new thread '{}' with stack base at {:x?}",
        thread.name.as_str(),
        stack_addr,
    );
    debug!("Thread: {:?}", thread);

    unsafe {
        initialize_stack(
            stack_page.start_address(),
            &thread,
            pointer as *const c_void,
        );
    }

    let boxed = Box::pin(thread);

    debug!("Thread addr: {:x?}", &*boxed);

    boxed
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

pub unsafe fn cleanup_thread(current_thread: *mut Thread) {
    debug!("Cleaning up thread {}", (*current_thread).name);

    SCHEDULER.as_mut().unwrap().thank_you_next();
}

global_asm!(include_str!("asm/threads.s"));
