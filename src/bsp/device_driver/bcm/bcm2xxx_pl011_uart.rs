use core::fmt::{self, Write};

use tock_registers::{
    interfaces::{Readable, Writeable},
    register_bitfields, register_structs,
    registers::{ReadOnly, ReadWrite, WriteOnly},
};

use crate::{
    bsp::device_driver::common::MMIODerefWrapper,
    console::interface,
    cpu,
    driver::interface::DeviceDriver,
    synchronization::{NullLock, interface::Mutex},
};

register_bitfields! {
    u32,
    FR [
        TXFE OFFSET(7) NUMBITS(1) [],
        TXFF OFFSET(5) NUMBITS(1) [],
        RXFE OFFSET(4) NUMBITS(1) [],
        BUSY OFFSET(3) NUMBITS(1) [],
    ],

    IBRD [
        BAUD_DIVINT OFFSET(0) NUMBITS(16) [],
    ],

    FBRD [
        BAUD_DIVFRAC OFFSET(0) NUMBITS(6) [],
    ],

    LCR_H [
        #[allow(clippy::enum_variant_names)]
        WLEN OFFSET(5) NUMBITS(2) [
            FiveBit = 0b00,
            SixBit = 0b01,
            SevenBit = 0b10,
            EightBit = 0b11,
        ],

        FEN OFFSET(4) NUMBITS(1) [
            FifosDisabled = 0,
            FifosEnabled = 1,
        ],
    ],

    CR [
        RXE OFFSET(9) NUMBITS(1) [
            Disabled = 0,
            Enabled = 1,
        ],

        TXE OFFSET(8) NUMBITS(1) [
            Disabled = 0,
            Enabled = 1,
        ],

        UARTEN OFFSET(0) NUMBITS(1) [
            Disabled = 0,
            Enabled = 1,
        ],
    ],

    ICR [
        ALL OFFSET(0) NUMBITS(11) [],
    ],
}

register_structs! {
    #[allow(non_snake_case)]
    pub RegisterBlock {
        (0x00 => DR: ReadWrite<u32>),
        (0x04 => _reserved1),
        (0x18 => FR: ReadOnly<u32, FR::Register>),
        (0x1c => _reserved2),
        (0x24 => IBRD: WriteOnly<u32, IBRD::Register>),
        (0x28 => FBRD: WriteOnly<u32, FBRD::Register>),
        (0x2c => LCR_H: WriteOnly<u32, LCR_H::Register>),
        (0x30 => CR: WriteOnly<u32, CR::Register>),
        (0x34 => _reserved3),
        (0x44 => ICR: WriteOnly<u32, ICR::Register>),
        (0x48 => @END),
    }
}

type Registers = MMIODerefWrapper<RegisterBlock>;

#[derive(PartialEq)]
enum BlockingMode {
    Blocking,
    NonBlocking,
}

struct PL011UartInner {
    registers: Registers,
    chars_written: usize,
    chars_read: usize,
}

impl PL011UartInner {
    pub const unsafe fn new(mmio_start_addr: usize) -> Self {
        Self {
            registers: unsafe { Registers::new(mmio_start_addr) },
            chars_written: 0,
            chars_read: 0,
        }
    }

    pub fn init(&mut self) {
        self.flush();

        self.registers.CR.set(0);

        self.registers.ICR.write(ICR::ALL::CLEAR);

        self.registers.IBRD.write(IBRD::BAUD_DIVINT.val(3));
        self.registers.FBRD.write(FBRD::BAUD_DIVFRAC.val(16));
        self.registers
            .LCR_H
            .write(LCR_H::WLEN::EightBit + LCR_H::FEN::FifosEnabled);

        self.registers
            .CR
            .write(CR::UARTEN::Enabled + CR::TXE::Enabled + CR::RXE::Enabled);
    }

    fn write_char(&mut self, c: char) {
        while self.registers.FR.matches_all(FR::TXFF::SET) {
            cpu::nop();
        }

        self.registers.DR.set(c as u32);

        self.chars_written += 1;
    }

    fn flush(&self) {
        while self.registers.FR.matches_all(FR::BUSY::SET) {
            cpu::nop();
        }
    }

    fn read_char_converting(&mut self, blocking_mode: BlockingMode) -> Option<char> {
        if self.registers.FR.matches_all(FR::RXFE::SET) {
            if blocking_mode == BlockingMode::NonBlocking {
                return None;
            }

            while self.registers.FR.matches_all(FR::RXFE::SET) {
                cpu::nop();
            }
        }

        let mut ret = self.registers.DR.get() as u8 as char;

        if ret == '\r' {
            ret = '\n';
        }
        self.chars_read += 1;

        Some(ret)
    }
}

impl fmt::Write for PL011UartInner {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            self.write_char(c);
        }

        Ok(())
    }
}

pub struct PL011Uart {
    inner: NullLock<PL011UartInner>,
}

impl PL011Uart {
    pub const COMPATIBLE: &'static str = "BCM PL011 UART";

    pub const unsafe fn new(mmio_start_addr: usize) -> Self {
        Self {
            inner: NullLock::new(unsafe { PL011UartInner::new(mmio_start_addr) }),
        }
    }
}

impl DeviceDriver for PL011Uart {
    fn compatible(&self) -> &'static str {
        Self::COMPATIBLE
    }

    unsafe fn init(&self) -> Result<(), &'static str> {
        self.inner.lock(|inner| inner.init());

        Ok(())
    }
}

impl interface::Write for PL011Uart {
    fn write_char(&self, c: char) {
        self.inner.lock(|inner| inner.write_char(c));
    }

    fn write_fmt(&self, args: fmt::Arguments) -> fmt::Result {
        self.inner.lock(|inner| inner.write_fmt(args))
    }

    fn flush(&self) {
        self.inner.lock(|inner| inner.flush());
    }
}

impl interface::Read for PL011Uart {
    fn read_char(&self) -> char {
        self.inner
            .lock(|inner| inner.read_char_converting(BlockingMode::Blocking).unwrap())
    }

    fn clear_rx(&self) {
        while self
            .inner
            .lock(|inner| inner.read_char_converting(BlockingMode::NonBlocking))
            .is_some()
        {}
    }
}

impl interface::Statistics for PL011Uart {
    fn chars_written(&self) -> usize {
        self.inner.lock(|inner| inner.chars_written)
    }

    fn chars_read(&self) -> usize {
        self.inner.lock(|inner| inner.chars_read)
    }
}

impl interface::All for PL011Uart {}
