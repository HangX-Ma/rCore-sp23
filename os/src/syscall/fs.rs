//! File and filesystem-related syscalls

// ch4
use crate::mm::translated_byte_buffer;
use crate::task::current_user_token;

// ch2 lab start
// use log::*;
// use crate::config::*;
// use crate::task::get_current_task;
// use crate::loader::{USER_STACK, get_base_i};

const FD_STDOUT: usize = 1;

// fn check_addr_legality(slice: &[u8]) -> Option<isize> {
//     let task_id = get_current_task();
//     let app_start = slice.as_ptr().addr();
//     let app_size = slice.len();
//     if !((app_start >= get_base_i(task_id) &&
//         app_start + app_size <= get_base_i(task_id) + APP_SIZE_LIMIT) ||
//         (app_start + app_size <= USER_STACK[task_id].get_sp() &&
//         app_start >= USER_STACK[task_id].get_sp() - USER_STACK_SIZE)) {
//         None
//     } else {
//         Some(app_size as isize)
//     }
// }
// ch2 lab end

/// write buf of length `len` to a file with `fd`
// pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
//     match fd {
//         FD_STDOUT => {
//             let slice = unsafe { core::slice::from_raw_parts(buf, len) };
//             match check_addr_legality(slice) {
//                 None => -1 as isize,
//                 Some(i_len) => {
//                     let str = core::str::from_utf8(slice).unwrap();
//                     print!("{}", str);
//                     i_len
//                 }
//             }
//         }
//         _ => {
//             // panic!("Unsupported fd in sys_write!");
//             -1 as isize
//         }
//     }
// }

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    match fd {
        FD_STDOUT => {
            let buffers = translated_byte_buffer(current_user_token(), buf, len);
            for buffer in buffers {
                print!("{}", core::str::from_utf8(buffer).unwrap());
            }
            len as isize
        },
        _ => {
            panic!("Unsupported fd in sys_write!");
        }
    }
}