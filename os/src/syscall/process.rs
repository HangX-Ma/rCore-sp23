//! App management syscalls

use crate::task::{
    change_program_brk,
    exit_current_and_run_next, 
    suspend_current_and_run_next,
    TaskStatus,
    current_user_token, 
    get_current_task_status, 
    get_current_task_syscall_times, 
    get_current_task_time_cost,
    get_current_task_page_table,
    create_new_map_area,
    unmap_consecutive_area
};

use crate::config::{MAX_SYSCALL_NUM, PAGE_SIZE, MAXVA};
use crate::timer::get_time_us;
use crate::mm::translated_byte_buffer;
use crate::mm::{VPNRange, VirtAddr, MapPermission};

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
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    suspend_current_and_run_next();
    0
}

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

pub fn sys_task_info(ti: *mut TaskInfo) -> isize {
    let dst_vec = translated_byte_buffer(
        current_user_token(),
        ti as *const u8, core::mem::size_of::<TaskInfo>()
    );
    let ref task_info = TaskInfo {
        status: get_current_task_status(),
        syscall_times: get_current_task_syscall_times(),
        time: get_current_task_time_cost(),
    };
    // println!("[kernel]: time {} syscall_time {}", task_info.time, task_info.syscall_times[super::SYSCALL_GET_TIME]);
    let src_ptr = task_info as *const TaskInfo;
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

/// port: page permission [2:0] X|W|R
pub fn sys_mmap(start: usize, len: usize, port: usize) -> isize {
    if start % PAGE_SIZE != 0 /* start need to be page aligned */ || 
        port & !0x7 != 0 /* other bits of port needs to be zero */ ||
        port & 0x7 ==0 /* No permission set, meaningless */ ||
        start >= MAXVA /* mapping range should be an legal address */ {
        return -1;
    }

    // check the range [start, start + len)
    let start_vpn = VirtAddr::from(start).floor();
    let end_vpn = VirtAddr::from(start + len).ceil();
    let vpns = VPNRange::new(start_vpn, end_vpn);
    for vpn in vpns {
       if let Some(pte) = get_current_task_page_table(vpn) {
            // we find a pte that has been mapped
            if pte.is_valid() {
                return -1;
            }
       }
    }
    // all ptes in range has pass the test
    create_new_map_area(
        start_vpn.into(),
        end_vpn.into(),
        MapPermission::from_bits_truncate((port << 1) as u8) | MapPermission::U
    );
    0
}


/// munmap the mapped virtual addresses
pub fn sys_munmap(start: usize, len: usize) -> isize {
    if start >= MAXVA || start % PAGE_SIZE != 0 {
        return -1;
    }
    // avoid undefined situation
    let mut mlen = len;
    if start > MAXVA - len {
        mlen = MAXVA - start;
    }
    unmap_consecutive_area(start, mlen)
}

/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
