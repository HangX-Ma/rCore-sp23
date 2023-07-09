//!Implementation of [`TaskManager`]

use super::{TaskControlBlock, TaskStatus};
use crate::config::BIG_STRIDE;
use crate::sync::UPSafeCell;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use lazy_static::*;

///A array of `TaskControlBlock` that is thread-safe
pub struct TaskManager {
    ready_queue: VecDeque<Arc<TaskControlBlock>>,
}

/// A simple FIFO scheduler.
impl TaskManager {
    /// Create an empty TaskManager
    pub fn new() -> Self {
        Self { ready_queue: VecDeque::new(), }
    }
    /// Add process back to ready queue
    pub fn add(&mut self, task: Arc<TaskControlBlock>) {
        self.ready_queue.push_back(task);
    }
    /// Take a process out of the ready queue
    pub fn fetch(&mut self) -> Option<Arc<TaskControlBlock>> {
        let mut min_index = 0;
        let mut min_stride = 0x7FFF_FFFF;
        for (idx, task) in self.ready_queue.iter().enumerate() {
            let inner = task.inner.exclusive_access();
            if inner.get_status() == TaskStatus::Ready {
                if inner.stride < min_stride {
                    min_stride = inner.stride;
                    min_index = idx;
                }
            }
        }

        if let Some(task) = self.ready_queue.get(min_index) {
            let mut inner = task.inner.exclusive_access();
            inner.stride += BIG_STRIDE / inner.priority;
        }
        self.ready_queue.remove(min_index)
    }
}

lazy_static! {
    pub static ref TASK_MANAGER: UPSafeCell<TaskManager> =
        unsafe { UPSafeCell::new(TaskManager::new()) };
}

/// Add process to ready queue
pub fn add_task(task: Arc<TaskControlBlock>) {
    TASK_MANAGER.exclusive_access().add(task);
}

/// Take a process out of the ready queue
pub fn fetch_task() -> Option<Arc<TaskControlBlock>> {
    TASK_MANAGER.exclusive_access().fetch()
}
