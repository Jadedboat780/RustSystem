use core::sync::atomic::{AtomicBool, Ordering::Relaxed};
use kernel_lib::{
    console::{ColorCode, Writer},
    serial::{self, SERIAL},
    serial_print,
    sync::IrqSpinLock,
};

pub static WRITER: IrqSpinLock<Writer> = IrqSpinLock::new(Writer::new(ColorCode::DEFAULT));

static SERIAL_MIRROR: AtomicBool = AtomicBool::new(false);

pub fn set_serial_mirror(enabled: bool) {
    SERIAL_MIRROR.store(enabled, Relaxed);
}

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    use core::fmt::Write;

    let _ = WRITER.lock().write_fmt(args);
    if SERIAL_MIRROR.load(Relaxed) {
        serial::_print(args);
    }
}

pub fn delete_char() {
    WRITER.lock().delete_char();
    if SERIAL_MIRROR.load(Relaxed) {
        serial_print!("\x08 \x08");
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::console::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

pub fn print_panic(info: &core::panic::PanicInfo) {
    use core::fmt::Write;

    if let Some(port) = SERIAL.as_ref() {
        let mut port = port.try_lock().unwrap_or_else(|| {
            unsafe { port.force_unlock() };
            port.lock()
        });
        let _ = writeln!(port, "{}", info);
    }

    let mut writer = WRITER.try_lock().unwrap_or_else(|| {
        unsafe { WRITER.force_unlock() };
        WRITER.lock()
    });
    let _ = write!(writer, "\n{}", info);
}
