// os/src/stack_btrace.r

use core::arch::asm;

pub fn btrace() {
    let mut fp: usize;
    unsafe {
        asm! (
            "mv {}, fp",
            out(reg) fp,
        );
    }
    println!("== STACK TRACE BEGIN");
    println!("=> start from fp addr: {:#x}", fp);
    while fp != 0 {
        let ra = (fp as *const usize).wrapping_sub(1);
        let prev_fp = (fp as *const usize).wrapping_sub(2);
        unsafe {
            println!("=> prev_fp:{:#x} ra:{:#x}", *ra, *prev_fp);
            fp = *prev_fp;
        }
    }
    println!("== STACK TRACE END");
}