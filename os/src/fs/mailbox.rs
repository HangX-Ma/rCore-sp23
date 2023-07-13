//! mail message between different processes

use crate::sync::UPSafeCell;
use crate::config::{MAX_MAIL_LENGTH, MAX_MESSAGE_NUM};


#[derive(Clone, Copy)]
pub struct Mail {
    pub data: [u8; MAX_MAIL_LENGTH],
    pub len: usize,
}

impl Mail {
    fn new() -> Self {
        Self { data: [0; MAX_MAIL_LENGTH], len: 0 }
    }
}

pub struct MailBox {
    pub buffer: UPSafeCell<MailBoxInner>,
}

impl MailBox {
    pub fn new() -> Self {
        unsafe {
            Self { buffer: UPSafeCell::new(MailBoxInner::new()) }
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum MailBoxStatus {
    Full,
    Empty,
    Normal,
}

pub struct MailBoxInner {
    pub arr: [Mail; MAX_MESSAGE_NUM],
    pub head: usize,
    pub tail: usize,
    pub status: MailBoxStatus,
}

impl MailBoxInner {
    pub fn new() -> Self {
        Self {
            arr: [Mail::new(); MAX_MESSAGE_NUM],
            head: 0,
            tail: 0,
            status: MailBoxStatus::Empty,
        }
    }

    pub fn is_full(&self) -> bool {
        self.status == MailBoxStatus::Full
    }

    pub fn is_empty(&self) -> bool {
        self.status == MailBoxStatus::Empty
    }

    pub fn available_read(&self) -> usize {
        if self.status == MailBoxStatus::Empty {
            0
        } else if self.tail > self.head {
            self.tail - self.head
        } else {
            self.tail + MAX_MAIL_LENGTH - self.head
        }
    }
    pub fn available_write(&self) -> usize {
        if self.status == MailBoxStatus::Full {
            0
        } else {
            MAX_MAIL_LENGTH - self.available_read()
        }
    }
}