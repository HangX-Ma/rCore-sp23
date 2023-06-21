use core::arch::asm;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn main() {
    let mut fp: u64;
    println!("=================== stack trace begin ==================");
    unsafe {
        asm! (
            "mov {}, rsp",
            out(reg) fp,
        );
    }
    println!("Initial stack frame pointer address: {:#x}", fp);
    println!("[return address]\t[prev frame address]");

    while fp != 0x01 {
        unsafe {
            println!("{:#x}\t\t{:#x}", *((fp - 8) as *mut u64),*((fp - 16) as *mut u64) );
            fp = *((fp - 16) as *mut u64);
        }
    }
    println!("=================== trace end ==================");

}
