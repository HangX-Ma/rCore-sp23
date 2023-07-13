//!Implementation of [`TaskManager`]

use super::{TaskControlBlock, TaskStatus};
use crate::config::BIG_STRIDE;
use crate::sync::UPSafeCell;
use alloc::collections::{BTreeMap, VecDeque};
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
    /// TASK_MANAGER instance through lazy_static!
    pub static ref TASK_MANAGER: UPSafeCell<TaskManager> =
        unsafe { UPSafeCell::new(TaskManager::new()) };
    /// PID2PCB instance (map of pid to pcb)
    pub static ref PID2TCB: UPSafeCell<BTreeMap<usize, Arc<TaskControlBlock>>> =
        unsafe { UPSafeCell::new(BTreeMap::new()) };
}

/// Add process to ready queue
pub fn add_task(task: Arc<TaskControlBlock>) {
    PID2TCB
        .exclusive_access()
        .insert(task.getpid(), Arc::clone(&task));
    TASK_MANAGER.exclusive_access().add(task);
}

/// Take a process out of the ready queue
pub fn fetch_task() -> Option<Arc<TaskControlBlock>> {
    TASK_MANAGER.exclusive_access().fetch()
}

/// Get process by pid
pub fn pid2task(pid: usize) -> Option<Arc<TaskControlBlock>> {
    let map = PID2TCB.exclusive_access();
    map.get(&pid).map(Arc::clone)
}

/// Remove item(pid, _some_pcb) from PDI2PCB map (called by exit_current_and_run_next)
pub fn remove_from_pid2task(pid: usize) {
    let mut map = PID2TCB.exclusive_access();
    if map.remove(&pid).is_none() {
        panic!("cannot find pid {} in pid2task!", pid);
    }
}

