//! App management syscalls
use core::time;

// use super::stats::*; // lab2-pro3
use crate::task::{
    exit_current_and_run_next, 
    suspend_current_and_run_next,
    TaskStatus,
    get_current_task_id,
    get_total_task_num,
};
use crate::config::MAX_SYSCALL_NUM;
use crate::timer::{
    get_time_ms,
    get_time_us
};
use crate::sync::UPSafeCell;
use lazy_static::*;

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

pub struct TaskInfo {
    inner: UPSafeCell<TaskInfoInner>,
}

pub struct TaskInfoInner {
    /// Task status in it's life cycle
    status: TaskStatus,
    /// The numbers of syscall called by task
    syscall_times: [u32; MAX_SYSCALL_NUM],
    /// Total running time of task, which consists of kernel time and user time
    time: usize,
}

lazy_static!(
    static ref TASK_INFO: TaskInfo = TaskInfo::new();
);

impl TaskInfo {
    fn new() -> Self {
        TaskInfo {
            inner: unsafe {
                UPSafeCell::new(TaskInfoInner {
                    status: TaskStatus::UnInit,
                    syscall_times: [0; MAX_SYSCALL_NUM],
                    time: 0,
                })
            }
        }
    }

    pub fn update(&self, syscall_id: usize) {
        let mut inner = self.inner.exclusive_access();
        inner.status = TaskStatus::Running; // keep it running because current task always running
        inner.syscall_times[syscall_id] += 1;
    }
} 

/// task exits and submit an exit code
pub fn sys_exit(exit_code: i32) -> ! {
    println!("[kernel] Application exited with code {}", exit_code);
    // stats_clear_and_print(); // lab2-pro3
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    suspend_current_and_run_next();
    0
}

// pub fn sys_get_time() -> isize {
//     get_time_ms() as isize
// }

pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    let us = get_time_us();
    unsafe {
        *ts = TimeVal {
            sec: us / 1_000_000,
            usec: us % 1_000_000,
        };
    }
    0
}

pub fn sys_task_info(ti: *mut TaskInfo) -> isize {

    -1
}

