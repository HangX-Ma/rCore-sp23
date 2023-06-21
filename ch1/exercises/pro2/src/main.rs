use core::arch::asm;

fn print_stack_trace_chain() {
    let fp: usize;
    println!("== STACK TRACE BEGIN");
    unsafe {
        asm! (
            "mov {fp}, rbp",
            fp = out(reg) fp,
        );
    }

    let mut fp = fp;
    for _ in 0..5 {
        println!(" == {:#p}", (fp) as *mut usize);
        fp = unsafe {
            (fp as *const usize).offset(0).read()
        };
    }
    println!("== STACK TRACE END");
}

fn finite_loop(num: i64) -> i64 {
    println!("Calling finite_loop({})", num);
    print_stack_trace_chain();
    if num == 0 {
        return 1;
    }
    return finite_loop(num - 1) * num;
}


fn main() {
    print_stack_trace_chain();
    println!("Output: {}", finite_loop(2));
}
