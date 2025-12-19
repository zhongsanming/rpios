use core::sync::atomic::{AtomicBool, Ordering};

use super::memory::map::mmio;
use crate::{
    bsp::device_driver::{GPIO, PL011Uart},
    console,
    driver::{DeviceDriverDescriptor, DriverManager},
};

pub(crate) static PL011_UART: PL011Uart = unsafe { PL011Uart::new(mmio::PL011_UART_START) };
static GPIO: GPIO = unsafe { GPIO::new(mmio::GPIO_START) };

fn post_init_uart() -> Result<(), &'static str> {
    console::register_console(&PL011_UART);
    Ok(())
}

fn post_init_gpio() -> Result<(), &'static str> {
    GPIO.map_pl011_uart();
    Ok(())
}

fn driver_uart() -> Result<(), &'static str> {
    let uart_descriptor = DeviceDriverDescriptor::new(&PL011_UART, Some(post_init_uart));
    DriverManager::global().register_driver(uart_descriptor);
    Ok(())
}

fn driver_gpio() -> Result<(), &'static str> {
    let gpio_descriptor = DeviceDriverDescriptor::new(&GPIO, Some(post_init_gpio));
    DriverManager::global().register_driver(gpio_descriptor);
    Ok(())
}

pub unsafe fn init() -> Result<(), &'static str> {
    static INIT_DONE: AtomicBool = AtomicBool::new(false);
    if INIT_DONE.load(Ordering::Relaxed) {
        return Err("Init already done");
    }

    driver_uart()?;
    driver_gpio()?;

    INIT_DONE.store(true, Ordering::Relaxed);
    Ok(())
}
