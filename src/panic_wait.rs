//! A panic handler that infinitely waits.

use core::panic::PanicInfo;

// private code
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unimplemented!()
}
