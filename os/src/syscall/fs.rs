//! File and filesystem-related syscalls

use crate::mm::{translated_byte_buffer, translated_str, UserBuffer};
use crate::task::{current_task, current_user_token};
use crate::fs::{open_file, OpenFlags, Stat, ROOT_INODE, OSInode, StatMode};
use core::any::Any;

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
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
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
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
        file.read(UserBuffer::new(translated_byte_buffer(token, buf, len))) as isize
    } else {
        -1
    }
}

pub fn sys_open(path: *const u8, flags: u32) -> isize {
    let task = current_task().unwrap();
    let token = current_user_token();
    let path = translated_str(token, path);
    if let Some(inode) = open_file(path.as_str(), OpenFlags::from_bits(flags).unwrap()) {
        let mut inner = task.inner_exclusive_access();
        let fd = inner.alloc_fd();
        inner.fd_table[fd] = Some(inode);
        fd as isize
    } else {
        -1
    }
}

pub fn sys_close(fd: usize) -> isize {
    let task = current_task().unwrap();
    let mut inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if inner.fd_table[fd].is_none() {
        return -1;
    }
    inner.fd_table[fd].take();
    0
}

/// YOUR JOB: Implement fstat.
pub fn sys_fstat(fd: usize, st: *mut Stat) -> isize {
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();

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
