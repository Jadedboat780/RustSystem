use crate::console::WRITER;
use kernel_lib::{serial_println, time};
use log::{Level, LevelFilter, Log, Metadata, Record};

struct KernelLogger;

static LOGGER: KernelLogger = KernelLogger;

impl Log for KernelLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &Record) {
        use core::fmt::Write;

        if !self.enabled(record.metadata()) {
            return;
        }
        let uptime = time::uptime();
        serial_println!(
            "[{:>5}.{:03}] {:<5} {}: {}",
            uptime.as_secs(),
            uptime.subsec_millis(),
            record.level(),
            record.target(),
            record.args()
        );
        if record.level() == Level::Error {
            let _ = writeln!(WRITER.lock(), "[{}] {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

pub fn init(level: LevelFilter) {
    if log::set_logger(&LOGGER).is_ok() {
        log::set_max_level(level);
    }
}

#[test_case]
fn test_logger_is_installed() {
    assert_eq!(log::max_level(), LevelFilter::Info);
    log::debug!("filtered out");
}
