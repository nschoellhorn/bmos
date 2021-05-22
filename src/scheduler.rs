use crate::cpu;
use crate::debug;
use crate::threading::Thread;
use alloc::boxed::Box;
use alloc::collections::VecDeque;
use alloc::vec::Vec;
use core::cell::RefCell;
use core::pin::Pin;

const TASK_MAX_TICKS: u8 = 10;

pub struct RoundRobinScheduler {
    current_task_ticks: u8,
    current_task: RefCell<Pin<Box<Thread>>>,
    tasks: VecDeque<Pin<Box<Thread>>>,
    dead_tasks: Vec<Pin<Box<Thread>>>,
}

impl RoundRobinScheduler {
    pub fn new(initial_task: Pin<Box<Thread>>) -> Self {
        Self {
            current_task_ticks: 0,
            current_task: RefCell::new(initial_task),
            tasks: VecDeque::new(),
            dead_tasks: Vec::new(),
        }
    }

    pub fn add_task(&mut self, task: Pin<Box<Thread>>) {
        self.tasks.push_back(task);
    }

    pub fn tick(&mut self) {
        self.current_task_ticks += 1;
        if self.current_task_ticks == TASK_MAX_TICKS {
            self.current_task_ticks = 0;
            self.next_task(true);
        }
    }

    fn next_task(&mut self, reschedule: bool) {
        let mut next_task = match self.tasks.pop_front() {
            Some(task) => task,
            None => return,
        };
        let new_addr = (&*next_task) as *const Thread;

        let mut old_task = self.current_task.replace(next_task);

        let old_addr = (&*old_task) as *const Thread;

        if reschedule && old_task.name != "main" {
            self.tasks.push_back(old_task);
        } else if old_task.name != "main" {
            self.dead_tasks.push(old_task);
        } else if old_task.name == "main" {
            drop(old_task);
        }

        unsafe {
            debug!("old: {:x?}, new: {:?}", old_addr, new_addr);
            cpu::switch_context(old_addr, new_addr);
        }
    }

    pub fn thank_you_next(&mut self) {
        self.current_task_ticks = 0;
        self.next_task(false);
    }
}
