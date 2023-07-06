//! App management syscalls

// use super::stats::*; // lab2-pro3
use crate::task::{
    change_program_brk,
    exit_current_and_run_next, 
    suspend_current_and_run_next,
    TaskStatus,
    current_user_token,
    // get_current_task_block,
};

use crate::config::MAX_SYSCALL_NUM;
use crate::timer::get_time_us;
use crate::mm::translated_byte_buffer;

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct TaskInfo {
    /// Task status in it's life cycle
    status: TaskStatus,
    /// The numbers of syscall called by task
    syscall_times: [u32; MAX_SYSCALL_NUM],
    /// Total running time of task, which consists of kernel time and user time
    time: usize,
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
    let dst_vec = translated_byte_buffer(
        current_user_token(),
        ts as *const u8, core::mem::size_of::<TimeVal>()
    );
    let ref time_val = TimeVal {
            sec: us / 1_000_000,
            usec: us % 1_000_000,
    };
    let src_ptr = time_val as *const TimeVal;
    for (idx, dst) in dst_vec.into_iter().enumerate() {
        let unit_len = dst.len();
        unsafe {
            dst.copy_from_slice(core::slice::from_raw_parts(
                src_ptr.wrapping_byte_add(idx * unit_len) as *const u8,
                unit_len)
            );
        }
    }
    0
}

// pub fn sys_task_info(ti: *mut TaskInfo) -> isize {
//     let task_block = get_current_task_block();
//     // println!("[kernel]: time {} syscall_time {}", task_block.kernel_time + task_block.user_time, task_block.syscall_times[SYSCALL_GET_TIME]);
//     unsafe {
//         *ti = TaskInfo {
//             status: task_block.task_status,
//             // syscall_times: task_block.syscall_times,
//             time: task_block.kernel_time + task_block.user_time,
//         };
//     }
//     0
// }

/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}

