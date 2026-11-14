use crate::sync::{Lazy, SpinLock};
use uart_16550::{Config, Uart16550Tty, backend::PioBackend};

pub static SERIAL: Lazy<SpinLock<Uart16550Tty<PioBackend>>> = Lazy::new(|| {
    let serial_port = unsafe { Uart16550Tty::new_port(0x3F8, Config::default()) }.expect("Serial port initialization failed");
    SpinLock::new(serial_port)
});

#[doc(hidden)]
pub fn _print(args: ::core::fmt::Arguments) {
    use core::fmt::Write;

    SERIAL.lock().write_fmt(args).expect("Printing to serial failed");
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
