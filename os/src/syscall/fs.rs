//! File and filesystem-related syscalls

// ch2 lab start
// use log::*;
// use crate::config::*;

const FD_STDOUT: usize = 1;

// fn check_addr_legality(slice: &[u8]) -> Option<isize> {
//     let app_start = slice.as_ptr().addr();
//     let app_size = slice.len();
//     if !((app_start >= APP_BASE_ADDRESS &&
//         app_start + app_size <= APP_BASE_ADDRESS + APP_SIZE_LIMIT) ||
//         (app_start + app_size <= unsafe { USER_STACK.get_sp().addr() } &&
//         app_start >= unsafe { USER_STACK.get_sp().addr() - USER_STACK_SIZE } )) {
//         None
//     } else {
//         Some(app_size as isize)
//     }
// }
// // ch2 lab end

// /// write buf of length `len` to a file with `fd`
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
//             error!("Unsupported fd in sys_write!");
//             -1 as isize
//         }
//     }
// }

/// write buf of length `len`  to a file with `fd`
/// 
pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    match fd {
        FD_STDOUT => {
            let slice = unsafe { core::slice::from_raw_parts(buf, len) };
            let str = core::str::from_utf8(slice).unwrap();
            print!("{}", str);
            len as isize
        }
        _ => {
            panic!("Unsupported fd in sys_write!");
        }
    }
}

