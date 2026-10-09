//! NS16550 UART console: output macros and polled input.
use crate::hw;
use core::fmt::{self, Write};

const UART0: usize = 0x1000_0000;

pub struct Uart;

pub fn put_byte(b: u8) {
    let base = UART0 as *mut u8;
    unsafe {
        while base.add(5).read_volatile() & (1 << 5) == 0 {} // wait: THR empty
        base.write_volatile(b);
    }
}

pub fn try_read() -> Option<u8> {
    let base = UART0 as *const u8;
    unsafe {
        if base.add(5).read_volatile() & 1 != 0 {
            Some(base.read_volatile())
        } else {
            None
        }
    }
}

impl Write for Uart {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for b in s.bytes() {
            if b == b'\n' {
                put_byte(b'\r');
            }
            put_byte(b);
        }
        Ok(())
    }
}

/// Interrupt-safe print so output from different tasks isn't garbled.
pub fn _print(args: fmt::Arguments) {
    hw::without_irq(|| {
        let _ = Uart.write_fmt(args);
    });
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => { $crate::console::_print(format_args!($($arg)*)) };
}
#[macro_export]
macro_rules! println {
    () => { $crate::print!("\n") };
    ($($arg:tt)*) => { $crate::print!("{}\n", format_args!($($arg)*)) };
}
