//! File trait & inode(dir, file, pipe, stdin, stdout)

mod inode;
mod stdio;

use crate::mm::UserBuffer;
use core::any::Any;

/// convert current type to &dyn Any
pub trait AnyConvertor {
    fn as_any(&self) -> &dyn Any;
}

impl<T: 'static> AnyConvertor for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// trait File for all file types
pub trait File: Send + Sync + AnyConvertor {
    /// the file readable?
    fn readable(&self) -> bool;
    /// the file writable?
    fn writable(&self) -> bool;
    /// read from the file to buf, return the number of bytes read
    fn read(&self, buf: UserBuffer) -> usize;
    /// write to the file from buf, return the number of bytes written
    fn write(&self, buf: UserBuffer) -> usize;
}

/// The stat of a inode
#[repr(C)]
#[derive(Debug)]
pub struct Stat {
    /// ID of device containing file
    pub dev: u64,
    /// inode number
    pub ino: u64,
    /// file type and mode
    pub mode: StatMode,
    /// number of hard links
    pub nlink: u32,
    /// unused pad
    pub pad: [u64; 7],
}

bitflags! {
    /// The mode of a inode
    /// whether a directory or a file
    pub struct StatMode: u32 {
        /// null
        const NULL  = 0;
        /// directory
        const DIR   = 0o040000;
        /// ordinary regular file
        const FILE  = 0o100000;
    }
}

pub use inode::{list_apps, open_file, OSInode, OpenFlags, ROOT_INODE};
pub use stdio::{Stdin, Stdout};
