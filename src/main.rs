//! 2. Once finished with architectural setup, the arch code calls `kernel_init()`.

#![allow(clippy::upper_case_acronyms)]
#![feature(format_args_nl)]
#![feature(trait_alias)]
#![no_main]
#![no_std]

use crate::driver::DriverManager;

mod bsp;
mod console;
mod cpu;
mod driver;
mod panic_wait;
mod print;
mod synchronization;

/// Early init code.
///
/// # Safety
///
/// - Only a single core must be active and running this function.
unsafe fn kernel_init() -> ! {
    // init bsp driver subsystem
    if let Err(x) = unsafe { bsp::driver::init() } {
        panic!("Error initializing BSP driver subsystem: {}", x);
    }

    // init all device drivers
    unsafe { DriverManager::global().init_drivers() };

    kernel_main()
}

fn kernel_main() -> ! {
    use console::console;

    println!(
        "[0] {} version {}!",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );
    println!("[1] Booting on: {}", bsp::board_name());
    println!("[2] Drivers loaded:");
    DriverManager::global().enumerate();

    println!("[3] Chars written: {}", console().chars_written());
    println!("[4] Echoing input now");

    console().clear_rx();
    loop {
        let c = console().read_char();
        console().write_char(c);
    }
}
