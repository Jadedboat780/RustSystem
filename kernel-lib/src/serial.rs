use crate::sync::{IrqSpinLock, Lazy};
use uart_16550::{Config, Uart16550Tty, backend::PioBackend, spec::registers::IER};

pub const COM1: u16 = 0x3F8;

pub type SerialPort = Uart16550Tty<PioBackend>;

pub static SERIAL: Lazy<Option<IrqSpinLock<SerialPort>>> = Lazy::new(|| {
    unsafe { Uart16550Tty::new_port(COM1, Config::default()) }
        .ok()
        .map(IrqSpinLock::new)
});

pub fn is_available() -> bool {
    SERIAL.is_some()
}

pub fn enable_receive_interrupt() {
    if let Some(port) = SERIAL.as_ref() {
        port.lock().inner_mut().enable_interrupts(IER::DATA_READY);
    }
}

pub fn read_byte() -> Option<u8> {
    SERIAL.as_ref()?.lock().inner_mut().try_receive_byte().ok()
}

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    use core::fmt::Write;

    if let Some(port) = SERIAL.as_ref() {
        let _ = port.lock().write_fmt(args);
    }
}

#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::serial::_print(format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(concat!($fmt, "\n"), $($arg)*));
}
