//! Task management implementation
//!
//! Everything about task management, like starting and switching tasks is
//! implemented here.
//!
//! A single global instance of [`TaskManager`] called `TASK_MANAGER` controls
//! all the tasks in the whole operating system.
//!
//! A single global instance of [`Processor`] called `PROCESSOR` monitors running
//! task(s) for each core.
//!
//! A single global instance of `PID_ALLOCATOR` allocates pid for user apps.
//!
//! Be careful when you see `__switch` ASM function in `switch.S`. Control flow around this function
//! might not be what you expect.
mod context;
mod pid;
mod manager;
mod processor;
mod switch;
#[allow(clippy::module_inception)]
mod task;

use crate::fs::{open_file, OpenFlags};
use alloc::sync::Arc;
use lazy_static::*;
pub use pid::{pid_alloc, KernelStack, PidHandle};
pub use manager::{fetch_task, add_task, TaskManager};
use switch::__switch;
pub use task::{TaskControlBlock, TaskControlBlockInner, TaskStatus};

pub use context::TaskContext;
pub use processor::{
    current_task, current_trap_cx, current_user_token, run_tasks, schedule, take_current_task,
    Processor,
};
use crate::config::MAX_SYSCALL_NUM;
use crate::mm::{VirtAddr, VirtPageNum, VPNRange, MapPermission, PageTableEntry};

/// Suspend the current 'Running' task and run the next task in task list.
pub fn suspend_current_and_run_next() {
    // There must be an application running.
    let task = take_current_task().unwrap();

    // ---- access current TCB exclusively
    let mut task_inner = task.inner_exclusive_access();
    let task_cx_ptr = &mut task_inner.task_cx as *mut TaskContext;
    // Change status to Ready
    task_inner.task_status = TaskStatus::Ready;
    // update kernel cost time
    task_inner.kernel_time += task_inner.update_checkpoint();
    drop(task_inner);
    // ---- release current PCB

    // push back to ready queue.
    add_task(task);
    // jump to scheduling cycle
    schedule(task_cx_ptr);
}

/// pid of usertests app in make run TEST=1
pub const IDLE_PID: usize = 0;

/// Exit the current 'Running' task and run the next task in task list.
pub fn exit_current_and_run_next(exit_code: i32) {
    // take from Processor
    let task = take_current_task().unwrap();

    let pid = task.getpid();
    if pid == IDLE_PID {
        println!(
            "[kernel] Idle process exit with exit_code {} ...",
            exit_code
        );
        panic!("All applications completed!");
    }

    // access current TCB exclusively
    let mut inner = task.inner_exclusive_access();
    // Change status to Zombie
    inner.task_status = TaskStatus::Zombie;
    // Record exit code
    inner.exit_code = exit_code;
    // do not move to its parent but under initproc

    // access initproc TCB exclusively
    {
        let mut initproc_inner = INITPROC.inner_exclusive_access();
        for child in inner.children.iter() {
            child.inner_exclusive_access().parent = Some(Arc::downgrade(&INITPROC));
            initproc_inner.children.push(child.clone());
        }
    }
    // release parent PCB

    inner.children.clear();
    // deallocate user space
    inner.memory_set.recycle_data_pages();

    // update kernel time cost
    inner.kernel_time += inner.update_checkpoint();
    drop(inner);
    // release current PCB
    // drop task manually to maintain rc correctly
    drop(task);

    // we do not have to save task context
    let mut _unused = TaskContext::zero_init();
    schedule(&mut _unused as *mut _);
}


//? ========= INITPROC ========= 
lazy_static! {
    /// Creation of initial process
    ///
    /// the name "initproc" may be changed to any other app name like "usertests",
    /// but we have user_shell, so we don't need to change it.
    pub static ref INITPROC: Arc<TaskControlBlock> = Arc::new({
        let inode = open_file("ch6b_initproc", OpenFlags::RDONLY).unwrap();
        let v = inode.read_all();
        TaskControlBlock::new(v.as_slice())
    });
}



///Add init process to the manager
pub fn add_initproc() {
    add_task(INITPROC.clone());
}


//* ch3-pro2
pub fn user_time_start() {
    let task = current_task().unwrap();
    let mut task_inner = task.inner_exclusive_access();
    task_inner.kernel_time += task_inner.update_checkpoint();
}

pub fn user_time_end() {
    let task = current_task().unwrap();
    let mut task_inner = task.inner_exclusive_access();
    task_inner.user_time += task_inner.update_checkpoint();
}

//* ch3,4-lab
// TaskControlBlock in chapter4 contains 'MemorySet' and other fields
// which cannot derive 'Clone' and 'Copy' traits. Therefore, we need to
// split the variables into separate parts
pub fn get_current_task_status() -> TaskStatus {
    let task = current_task().unwrap();
    let task_inner = task.inner_exclusive_access();
    task_inner.get_status()
}

pub fn get_current_task_time_cost() -> usize {
    let task = current_task().unwrap();
    let task_inner = task.inner_exclusive_access();
    task_inner.user_time + task_inner.kernel_time
}

pub fn get_current_task_syscall_times() -> [u32; MAX_SYSCALL_NUM] {
    let task = current_task().unwrap();
    let task_inner = task.inner_exclusive_access();
    task_inner.syscall_times
}

pub fn update_task_syscall_times(syscall_id: usize) {
    let task = current_task().unwrap();
    let mut task_inner = task.inner_exclusive_access();
    task_inner.syscall_times[syscall_id] += 1;
}

// //* ch4-lab2, mmap, munmap
pub fn get_current_task_page_table(vpn: VirtPageNum) -> Option<PageTableEntry> {
    let task = current_task().unwrap();
    let task_inner = task.inner_exclusive_access();
    task_inner.memory_set.translate(vpn)
}

pub fn create_new_map_area(start_va: VirtAddr, end_va: VirtAddr, perm: MapPermission) {
    let task = current_task().unwrap();
    let mut task_inner = task.inner_exclusive_access();
    task_inner.memory_set.insert_framed_area(start_va, end_va, perm);
}

pub fn unmap_consecutive_area(start: usize, len: usize) -> isize {
    let task = current_task().unwrap();
    let mut task_inner = task.inner_exclusive_access();
    let start_vpn = VirtAddr::from(start).floor();
    let end_vpn = VirtAddr::from(start + len).ceil();
    let vpns = VPNRange::new(start_vpn, end_vpn);
    for vpn in vpns {
        if let Some(pte) = task_inner.memory_set.translate(vpn) {
            if !pte.is_valid() {
                return -1;
            }
            task_inner.memory_set.get_page_table().unmap(vpn);
        } else {
            // Also unmapped if no PTE found
            return -1;
        }
    }
    0
}
