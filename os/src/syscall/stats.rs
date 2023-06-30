        
// lab2-pro3
// use hashbrown::HashMap;
// static mut SYS_STATS: HashMap<usize, usize> = HashMap::new();

// pub fn stats_update(syscall_id: usize) {
//     unsafe {
//         if !SYS_STATS.contains_key(&syscall_id) {
//             SYS_STATS.insert(syscall_id, 1);
//         } else {
//             // if we have add the syscall, we need to update its invoked times
//             let (id, num) = SYS_STATS.get_key_value_mut(&syscall_id).unwrap();
//             *num += 1;
//         }
//     }
// }

// pub fn stats_clear_and_print() {
//     unsafe {
//         if SYS_STATS.is_empty() {
//             println!("[kernel] No syscall is invoked");
//         } else {
//             for (id, num) in SYS_STATS {
//                 println!("[kernel] syscall id '{}' is invoked {} times", id, num);
//             }
//         }
//         SYS_STATS.clear();
//     }
// }