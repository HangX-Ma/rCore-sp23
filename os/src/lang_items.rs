// os/src/lang_items.rs
use core::panic::PanicInfo;
use crate::println;
use crate::sbi::shutdown;
use crate::stack_btrace::btrace;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        println!("Panicked at {}:{} {}",
            location.file(),
            location.line(),
            info.message().unwrap()
            
        );
    } else {
        println!("Panicked: {}", info.message().unwrap());
    }
    btrace(); // ch2-lab feature
    shutdown(true)
}