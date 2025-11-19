//! A panic handler that infinitely waits.

use crate::cpu;
use core::panic::PanicInfo;

// private code
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    cpu::wait_forever()
}
