//! The panic handler

use crate::sbi::shutdown;
use core::panic::PanicInfo;

use crate::stack_trace;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        println!(
            "[kernel] Panicked at {}:{} {}",
            location.file(),
            location.line(),
            info.message()
        );
        
        stack_trace();
        
    } else {
        println!("[kernel] Panicked: {}", info.message());
    }
    shutdown()
}
