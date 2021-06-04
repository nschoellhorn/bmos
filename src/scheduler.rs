use alloc::collections::VecDeque;
use alloc::sync::Arc;
use crate::debug;

use x86_64::VirtAddr;

use crate::cpu;
use crate::task::Thread;

pub struct Scheduler {
    current_task: Arc<Thread>,
    ready_queue: VecDeque<Arc<Thread>>,
}

impl Scheduler {
    pub fn new(init_task: Arc<Thread>) -> Self {
        Scheduler {
            current_task: init_task,
            ready_queue: VecDeque::new(),
        }
    }

    pub unsafe fn init(&self) {
        cpu::init_switch(&*self.current_task);
    }

    pub fn add_task(&mut self, task: Thread) {
        self.ready_queue.push_back(Arc::new(task));
    }

    pub fn current_stack(&self) -> VirtAddr {
        VirtAddr::new((&(self.current_task.stack[0]) as *const _) as u64)
    }

    /// This function is the core of the scheduling algorithm.
    /// It gives back the current/"previous" task and task that should be run next.
    /// It also makes sure to re-schedule the previous task again at the end of the queue.
    pub fn pick_next(&mut self) -> Option<(Arc<Thread>, Arc<Thread>)> {
        if self.ready_queue.len() > 0 {
            let next = self.ready_queue.pop_front().unwrap();

            let current = Arc::clone(&self.current_task);
            self.current_task = Arc::clone(&next);

            self.ready_queue.push_back(Arc::clone(&current));

            //debug!("Current Thread: {}, Next Thread: {}", current.name.as_str(), next.name.as_str());

            return Some((current, next));
        }

        None
    }
}
