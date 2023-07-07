//! Task management implementation
//!
//! Everything about task management, like starting and switching tasks is
//! implemented here.
//!
//! A single global instance of [`TaskManager`] called `TASK_MANAGER` controls
//! all the tasks in the operating system.
//!
//! Be careful when you see `__switch` ASM function in `switch.S`. Control flow around this function
//! might not be what you expect.

mod context;
mod switch;

#[allow(clippy::module_inception)]
mod task;

use core::usize;

use crate::config::MAX_SYSCALL_NUM;
use crate::loader::{get_app_data, get_num_app};
use crate::mm::{VirtPageNum, PageTableEntry, VirtAddr, MapPermission, VPNRange};
use crate::sbi::shutdown;
use crate::sync::UPSafeCell;
use crate::trap::TrapContext;
use alloc::vec::Vec;
use lazy_static::*;
use switch::__switch;
pub use task::{TaskControlBlock, TaskStatus};
pub use context::TaskContext;
pub use crate::timer::{get_time_ms, get_time_us};

/// The task manager, where all the tasks are managed.
///
/// Functions implemented on `TaskManager` deals with all task state transitions
/// and task context switching. For convenience, you can find wrappers around it
/// in the module level.
///
/// Most of `TaskManager` are hidden behind the field `inner`, to defer
/// borrowing checks to runtime. You can see examples on how to use `inner` in
/// existing functions on `TaskManager`.
pub struct TaskManager {
    /// total number of tasks
    num_app: usize,
    /// use inner value to get mutable access
    inner: UPSafeCell<TaskManagerInner>,
}

/// Inner of Task Manager
pub struct TaskManagerInner {
    /// task list
    tasks: Vec<TaskControlBlock>,
    /// id of current `Running` task
    current_task: usize,
    /// the number of tasks that have not exit
    alive_task_num: usize,
    /// record time point
    checkpoint: usize,
}

/// ch3-pro2
impl TaskManagerInner {
    /// update checkpoint and return the diff time
    fn update_checkpoint(&mut self) -> usize {
        let prev_point = self.checkpoint;
        self.checkpoint = get_time_ms();
        return self.checkpoint - prev_point;
    }
}

lazy_static! {
    /// a `TaskManager` global instance through lazy_static!
    pub static ref TASK_MANAGER: TaskManager = {
        let num_app = get_num_app();
        println!("[kernel] Start initializing TASK_MANAGER, num_app: {}", num_app);
        let mut tasks: Vec<TaskControlBlock> = Vec::new();
        for i in 0..num_app {
            tasks.push(TaskControlBlock::new(get_app_data(i), i));
        }
        TaskManager {
            num_app,
            inner: unsafe {
                UPSafeCell::new(TaskManagerInner {
                    tasks,
                    current_task: 0,
                    alive_task_num: num_app,
                    checkpoint: 0,
                })
            },
        }
    };
}

impl TaskManager {
    /// Run the first task in task list.
    ///
    /// Generally, the first task in task list is an idle task (we call it zero process later).
    /// But in ch3, we load apps statically, so the first task is a real app.
    fn run_first_task(&self) -> ! {
        let mut inner = self.inner.exclusive_access();
        let task0 = &mut inner.tasks[0];
        task0.task_status = TaskStatus::Running;
        let next_task_cx_ptr = &task0.task_cx as *const TaskContext;
        inner.update_checkpoint();
        drop(inner);
        let mut _unused = TaskContext::zero_init();
        // before this, we should drop local variables that must be dropped manually
        unsafe {
            __switch(&mut _unused as *mut TaskContext, next_task_cx_ptr);
        }
        panic!("unreachable in run_first_task!");
    }

    /// Change the status of current `Running` task into `Ready`.
    fn mark_current_suspended(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        // ch3-pro1
        // if inner.alive_task_num > 1 {
            // println!("[kernel] task {} suspended", current);
        // }
        //* ch3-pro2
        inner.tasks[current].kernel_time += inner.update_checkpoint();
        inner.tasks[current].task_status = TaskStatus::Ready;
    }

    /// Change the status of current `Running` task into `Exited`.
    fn mark_current_exited(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        //* ch3-pro1, 2
        inner.tasks[current].kernel_time += inner.update_checkpoint();
        println!("[kernel] task {} exited, cost in kernel {} ms and cost in user {} ms",
            current, inner.tasks.get(current).unwrap().kernel_time, inner.tasks.get(current).unwrap().kernel_time);
        inner.tasks[current].task_status = TaskStatus::Exited;
        inner.alive_task_num -= 1;
    }

    /// Find next task to run and return task id.
    ///
    /// In this case, we only return the first `Ready` task in task list.
    fn find_next_task(&self) -> Option<usize> {
        let inner = self.inner.exclusive_access();
        let current = inner.current_task;
        (current + 1..current + self.num_app + 1)
            .map(|id| id % self.num_app)
            .find(|id| inner.tasks[*id].task_status == TaskStatus::Ready)
    }

    /// Switch current `Running` task to the task we have found,
    /// or there is no `Ready` task and we can exit with all applications completed
    fn run_next_task(&self) {
        if let Some(next) = self.find_next_task() {
            let mut inner = self.inner.exclusive_access();
            let current = inner.current_task;
            inner.tasks[next].task_status = TaskStatus::Running;
            inner.current_task = next;
            let current_task_cx_ptr = &mut inner.tasks[current].task_cx as *mut TaskContext;
            let next_task_cx_ptr = &inner.tasks[next].task_cx as *const TaskContext;
            drop(inner);
            // before this, we should drop local variables that must be dropped manually
            // ch3-pro1
            if current != next {
                // println!("[kernel] task switch from {} to {}", current, next);
                unsafe {
                    __switch(current_task_cx_ptr, next_task_cx_ptr);
                }
            }
            // go back to user mode
        } else {
            println!("All applications completed!");
            shutdown(false);
        }
    }
    // ch4
    /// Get the current 'Running' task's token.
    fn get_current_token(&self) -> usize {
        let inner = self.inner.exclusive_access();
        inner.tasks[inner.current_task].get_user_token()
    }

    /// Get the current 'Running' task's trap contexts.
    fn get_current_trap_cx(&self) -> &'static mut TrapContext {
        let inner = self.inner.exclusive_access();
        inner.tasks[inner.current_task].get_trap_cx()
    }

    /// Change the current 'Running' task's program break
    pub fn change_current_program_brk(&self, size: i32) -> Option<usize> {
        let mut inner = self.inner.exclusive_access();
        let cur = inner.current_task;
        inner.tasks[cur].change_program_brk(size)
    }

    //* ch3-pro2 start
    /// record the kernel time, now start to record the user time
    pub fn user_time_start(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].kernel_time += inner.update_checkpoint();
    }

    /// record the user time, now start to record the kernel time
    pub fn user_time_end(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].user_time += inner.update_checkpoint();
    }
    //* ch3-pro2 end
}

/// run first task
pub fn run_first_task() {
    TASK_MANAGER.run_first_task();
}

/// rust next task
fn run_next_task() {
    TASK_MANAGER.run_next_task();
}

/// suspend current task
fn mark_current_suspended() {
    TASK_MANAGER.mark_current_suspended();
}

/// exit current task
fn mark_current_exited() {
    TASK_MANAGER.mark_current_exited();
}

/// suspend current task, then run next task
pub fn suspend_current_and_run_next() {
    mark_current_suspended();
    run_next_task();
}

/// exit current task,  then run next task
pub fn exit_current_and_run_next() {
    mark_current_exited();
    run_next_task();
}

//* ch4
pub fn current_user_token() -> usize {
    TASK_MANAGER.get_current_token()
}

pub fn current_trap_cx() -> &'static mut TrapContext {
    TASK_MANAGER.get_current_trap_cx()
}
/// Change the current 'Running' task's program break
pub fn change_program_brk(size: i32) -> Option<usize> {
    TASK_MANAGER.change_current_program_brk(size)
}

//* ch3-pro2
pub fn user_time_start() {
    TASK_MANAGER.user_time_start();
}

pub fn user_time_end() {
    TASK_MANAGER.user_time_end();
}

//* ch3,4-lab
// TaskControlBlock in chapter4 contains 'MemorySet' and other fields
// which cannot derive 'Clone' and 'Copy' traits. Therefore, we need to
// split the variables into separate parts
pub fn get_current_task_status() -> TaskStatus {
    let inner = TASK_MANAGER.inner.exclusive_access();
    let current = inner.current_task;
    inner.tasks.get(current).unwrap().task_status
}

pub fn get_current_task_time_cost() -> usize {
    let inner = TASK_MANAGER.inner.exclusive_access();
    let current = inner.current_task;
    let task_block = inner.tasks.get(current).unwrap();
    task_block.kernel_time + task_block.user_time
}

pub fn get_current_task_syscall_times() -> [u32; MAX_SYSCALL_NUM] {
    let inner = TASK_MANAGER.inner.exclusive_access();
    let current = inner.current_task;
    inner.tasks.get(current).unwrap().syscall_times
}

pub fn update_task_syscall_times(syscall_id: usize) {
    let mut inner = TASK_MANAGER.inner.exclusive_access();
    let current = inner.current_task;
    inner.tasks[current].syscall_times[syscall_id] += 1;
}

//* ch4-lab2, mmap, munmap
pub fn get_current_task_page_table(vpn: VirtPageNum) -> Option<PageTableEntry> {
    let inner = TASK_MANAGER.inner.exclusive_access();
    let current = inner.current_task;
    inner.tasks[current].memory_set.translate(vpn)
}

pub fn create_new_map_area(start_va: VirtAddr, end_va: VirtAddr, perm: MapPermission) {
    let mut inner = TASK_MANAGER.inner.exclusive_access();
    let current = inner.current_task;
    inner.tasks[current].memory_set.insert_framed_area(start_va, end_va, perm);
}

pub fn unmap_consecutive_area(start: usize, len: usize) -> isize {
    let mut inner = TASK_MANAGER.inner.exclusive_access();
    let current = inner.current_task;
    let start_vpn = VirtAddr::from(start).floor();
    let end_vpn = VirtAddr::from(start + len).ceil();
    let vpns = VPNRange::new(start_vpn, end_vpn);
    for vpn in vpns {
        if let Some(pte) = inner.tasks[current].memory_set.translate(vpn) {
            if !pte.is_valid() {
                return -1;
            }
            inner.tasks[current].memory_set.get_page_table().unmap(vpn);
        } else {
            // Also unmapped if no PTE found
            return -1;
        }
    }
    0
}
