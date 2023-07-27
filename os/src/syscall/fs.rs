//! File and filesystem-related syscalls

use crate::mm::{translated_byte_buffer, translated_refmut, translated_str, UserBuffer};
use crate::task::{current_process, current_task, current_user_token};
#[allow(unused)]
use crate::fs::{make_pipe, open_file, OpenFlags, Stat, ROOT_INODE, OSInode, StatMode, MailBoxStatus};
#[allow(unused)]
use crate::config::{MAX_MAIL_LENGTH, MAX_MESSAGE_NUM};
use core::any::Any;
use alloc::sync::Arc;

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    trace!(
        "kernel:pid[{}] sys_write",
        current_task().unwrap().process.upgrade().unwrap().getpid()
    );
    let token = current_user_token();
    let process = current_process();
    let inner = process.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        if !file.writable() {
            return -1;
        }
        let file = file.clone();
        // release current task TCB manually to avoid multi-borrow
        drop(inner);
        file.write(UserBuffer::new(translated_byte_buffer(token, buf, len))) as isize
    } else {
        -1
    }
}

pub fn sys_read(fd: usize, buf: *const u8, len: usize) -> isize {
    trace!(
        "kernel:pid[{}] sys_read",
        current_task().unwrap().process.upgrade().unwrap().getpid()
    );
    let token = current_user_token();
    let process = current_process();
    let inner = process.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        let file = file.clone();
        if !file.readable() {
            return -1;
        }
        // release current task TCB manually to avoid multi-borrow
        drop(inner);
        trace!("kernel: sys_read .. file.read");
        file.read(UserBuffer::new(translated_byte_buffer(token, buf, len))) as isize
    } else {
        -1
    }
}

pub fn sys_open(path: *const u8, flags: u32) -> isize {
    trace!(
        "kernel:pid[{}] sys_open",
        current_task().unwrap().process.upgrade().unwrap().getpid()
    );
    let process = current_process();
    let token = current_user_token();
    let path = translated_str(token, path);
    if let Some(inode) = open_file(path.as_str(), OpenFlags::from_bits(flags).unwrap()) {
        let mut inner = process.inner_exclusive_access();
        let fd = inner.alloc_fd();
        inner.fd_table[fd] = Some(inode);
        fd as isize
    } else {
        -1
    }
}

pub fn sys_close(fd: usize) -> isize {
    trace!(
        "kernel:pid[{}] sys_close",
        current_task().unwrap().process.upgrade().unwrap().getpid()
    );
    let process = current_process();
    let mut inner = process.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if inner.fd_table[fd].is_none() {
        return -1;
    }
    inner.fd_table[fd].take();
    0
}

pub fn sys_pipe(pipe: *mut usize) -> isize {
    trace!(
        "kernel:pid[{}] sys_pipe",
        current_task().unwrap().process.upgrade().unwrap().getpid()
    );
    let process = current_process();
    let token = current_user_token();
    let mut inner = process.inner_exclusive_access();
    let (pipe_read, pipe_write) = make_pipe();
    let read_fd = inner.alloc_fd();
    inner.fd_table[read_fd] = Some(pipe_read);
    let write_fd = inner.alloc_fd();
    inner.fd_table[write_fd] = Some(pipe_write);
    *translated_refmut(token, pipe) = read_fd;
    *translated_refmut(token, unsafe { pipe.add(1) }) = write_fd;
    0
}

pub fn sys_dup(fd: usize) -> isize {
    trace!(
        "kernel:pid[{}] sys_dup",
        current_task().unwrap().process.upgrade().unwrap().getpid()
    );
    let process = current_process();
    let mut inner = process.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if inner.fd_table[fd].is_none() {
        return -1;
    }
    let new_fd = inner.alloc_fd();
    inner.fd_table[new_fd] = Some(Arc::clone(inner.fd_table[fd].as_ref().unwrap()));
    new_fd as isize
}

/// YOUR JOB: Implement fstat.
pub fn sys_fstat(fd: usize, st: *mut Stat) -> isize {
    trace!(
        "kernel:pid[{}] sys_fstat NOT IMPLEMENTED",
        current_task().unwrap().process.upgrade().unwrap().getpid()
    );
    let process = current_process();
    let inner = process.inner_exclusive_access();

    // check legality
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if inner.fd_table[fd].is_none() {
        return -1;
    }

    let ino: u64;
    let nlink: u32;
    if let Some(file_node) = &inner.fd_table[fd] {
        let any: &dyn Any = file_node.as_any();
        let os_node = any.downcast_ref::<OSInode>().unwrap();
        ino = os_node.get_inode_id();
        let (block_id, block_offset) = os_node.get_inode_pos();
        nlink = ROOT_INODE.get_link_num(block_id, block_offset);
    } else {
        return -1;
    }

    let stat = &Stat {
        dev: 0,
        ino: ino,
        mode: StatMode::FILE,
        nlink: nlink,
        pad: [0;7],
    };

    // copy data from kernel space to user space
    let token = inner.get_user_token();
    let st = translated_byte_buffer(token, st as *const u8, core::mem::size_of::<Stat>());
    let stat_ptr = stat as *const _ as *const u8;
    for (idx, byte) in st.into_iter().enumerate() {
        unsafe {
            byte.copy_from_slice(core::slice::from_raw_parts(stat_ptr.wrapping_byte_add(idx), byte.len()));
        }
    }
    0
}

/// YOUR JOB: Implement linkat.
pub fn sys_linkat(old_name: *const u8, new_name: *const u8) -> isize {
    let token = current_user_token();
    let old = translated_str(token, old_name);
    let new = translated_str(token, new_name);
    println!("link {} to {}", new , old);
    if old.as_str() != new.as_str() {
        if let Some(_) = ROOT_INODE.link(old.as_str(), new.as_str()) {
            return 0;
        }
    }
    -1
}

/// YOUR JOB: Implement unlinkat.
pub fn sys_unlinkat(name: *const u8) -> isize {
    let token = current_user_token();
    let name = translated_str(token, name);
    if let Some(inode) = ROOT_INODE.find(name.as_str()) {
        if ROOT_INODE.get_link_num(inode.block_id, inode.block_offset) == 1 {
            // clear data if only one link exists
            inode.clear();
        }
        return ROOT_INODE.unlink(name.as_str());
    }
    -1
}

// #[allow(unused)]
// pub fn sys_mail_read(buf: *mut u8, len: usize) -> isize {
//     if len == 0 {
//         return 0;
//     }
//     let process = current_process();
//     let inner = process.inner_exclusive_access();
//     let token = inner.get_user_token();
//     let mut mailbox_inner = inner.mailbox.buffer.exclusive_access();
//     if mailbox_inner.is_empty() {
//         return -1;
//     }
//     let mailbox_head = mailbox_inner.head;
//     // the truncated mail length
//     let mlen = len.min(mailbox_inner.arr[mailbox_head].len);
//     let dst_vec = translated_byte_buffer(token, buf, mlen);
//     let src_ptr = mailbox_inner.arr[mailbox_head].data.as_ptr();
//     for (idx, dst) in dst_vec.into_iter().enumerate() {
//         unsafe {
//             dst.copy_from_slice(
//                 core::slice::from_raw_parts(
//                     src_ptr.wrapping_add(idx) as *const u8,
//                     core::mem::size_of::<u8>()
//                     )
//             );
//         }
//     }
//     mailbox_inner.status = MailBoxStatus::Normal;
//     mailbox_inner.head = (mailbox_head + 1) % MAX_MAIL_LENGTH;
//     if mailbox_inner.head == mailbox_inner.tail {
//         mailbox_inner.status = MailBoxStatus::Empty;
//     }
//     0
// }

// #[allow(unused)]
// pub fn sys_mail_write(pid: usize, buf: *mut u8, len: usize) -> isize {
//     if core::ptr::null() == buf {
//         return -1;
//     }
//     if len == 0 {
//         return 0;
//     }
//     if let Some(target_task) = pid2task(pid) {
//         let target_task_ref = target_task.inner_exclusive_access();
//         let token = target_task_ref.get_user_token();
//         let mut mailbox_inner = target_task_ref.mailbox.buffer.exclusive_access();
//         if mailbox_inner.is_full() {
//             return -1;
//         }
//         let mailbox_tail = mailbox_inner.tail;
//         mailbox_inner.status = MailBoxStatus::Normal;
//         // the truncated mail length
//         let mlen = len.min(MAX_MAIL_LENGTH);
//         // prepare source data
//         let src_vec = translated_byte_buffer(token, buf, mlen);
//         // copy from source to dst
//         for (idx, src) in src_vec.into_iter().enumerate() {
//             unsafe {
//                 mailbox_inner.arr[mailbox_tail].data[idx..=idx].copy_from_slice(
//                     core::slice::from_raw_parts(
//                             src.as_ptr(),
//                             core::mem::size_of::<u8>()
//                             )
//                     );
//             }
//         }
//         // store the mail length
//         mailbox_inner.arr[mailbox_tail].len = mlen;

//         mailbox_inner.tail = (mailbox_tail + 1) % MAX_MESSAGE_NUM;
//         if mailbox_inner.tail == mailbox_inner.head {
//             mailbox_inner.status = MailBoxStatus::Full;
//         }
//         return 0;
//     }
//     -1
// }