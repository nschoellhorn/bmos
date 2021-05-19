use crate::cpu;
use crate::threading::Thread;
use alloc::collections::VecDeque;
use core::cell::RefCell;

const TASK_MAX_TICKS: u8 = 10;

pub struct RoundRobinScheduler {
    current_task_ticks: u8,
    current_task: RefCell<Thread>,
    tasks: VecDeque<Thread>,
}

impl RoundRobinScheduler {
    pub fn new(initial_task: Thread) -> Self {
        Self {
            current_task_ticks: 0,
            current_task: RefCell::new(initial_task),
            tasks: VecDeque::new(),
        }
    }

    pub fn add_task(&mut self, task: Thread) {
        self.tasks.push_back(task);
    }

    pub fn tick(&mut self) {
        self.current_task_ticks += 1;
        if self.current_task_ticks == TASK_MAX_TICKS {
            self.current_task_ticks = 0;
            self.next_task();
        }
    }

    fn next_task(&mut self) {
        let next_task = match self.tasks.pop_front() {
            Some(task) => task,
            None => return,
        };
        let old_task = self.current_task.replace(next_task);

        self.tasks.push_back(old_task);
        let old_task_ref = self.tasks.get_mut(self.tasks.len() - 1).unwrap();

        unsafe {
            cpu::switch_context(old_task_ref, self.current_task.as_ptr().as_ref().unwrap());
        }
    }
}
