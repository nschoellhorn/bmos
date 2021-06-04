use crate::{
    keyboard::{KeyEvent, KEYBOARD_REGISTRY},
    task,
};
use alloc::collections::VecDeque;
use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::instructions::interrupts::without_interrupts;

pub enum SystemEvent {
    KeyboardEvent(KeyEvent),
}

lazy_static! {
    pub static ref EVENT_QUEUE: Mutex<VecDeque<SystemEvent>> =
        Mutex::new(VecDeque::with_capacity(32));
}

pub fn init() {
    task::spawn_thread(|| {
        // Each time this thread is scheduled, we process up to 10 events, then we yield
        loop {
            without_interrupts(|| {
                let mut queue = EVENT_QUEUE.lock();
                if !queue.is_empty() {
                    for _ in 0..10 {
                        let event = queue.pop_front().unwrap();

                        match event {
                            SystemEvent::KeyboardEvent(event) => unsafe {
                                if let Some(registry) = &KEYBOARD_REGISTRY {
                                    registry.dispatch_event(event);
                                }
                            },
                        }

                        if queue.is_empty() {
                            break;
                        }
                    }
                }
            });

            unsafe {
                asm!("hlt");
            }
        }
    });
}
