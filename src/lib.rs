#![no_std]
#![cfg_attr(test, no_main)]
#![feature(abi_x86_interrupt)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

pub mod allocator;
pub mod commands;
pub mod interrupts;
pub mod keyboard;
pub mod syscalls;

use bootloader_api::{BootloaderConfig, config::Mapping};
use kernel_lib::{
    gdt, pit,
    sync::spin_lock::SpinLock,
    time::sleep_cycles,
    vga::{
        colors::{Color, ColorCode},
        writer::Writer,
    },
};

pub const BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};

pub fn init() {
    gdt::init();
    pit::init();
    interrupts::init_idt();
    unsafe {
        let mut pics = interrupts::PICS.lock();
        pics.initialize();
        pics.write_masks(0b1111_1100, 0b1111_1111);
    }
    x86_64::instructions::interrupts::enable();
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    // Stop interrupts while we're printing
    interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).expect("Printing failed");
    });
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

pub static WRITER: SpinLock<Writer> = SpinLock::new(Writer::new(0, ColorCode::new(Color::Pink, Color::Black)));

pub fn print_logo(row: usize, col: usize) {
    const ASCII_LOGO: &str = r"
         ____            _     ____            _
        |  _ \ _   _ ___| |_  / ___| _   _ ___| |_ ___ _ __ ___
        | |_) | | | / __| __| \___ \| | | / __| __/ _ \ '_ ` _ \
        |  _ <| |_| \__ \ |_   ___) | |_| \__ \ ||  __/ | | | | |
        |_| \_\\__,_|___/\__| |____/ \__, |___/\__\___|_| |_| |_|
                                     |___/
";
    println!("{}", ASCII_LOGO);

    let dots = [b'.', b'.', b'.'];

    for _ in 0..5 {
        for i in 0..dots.len() {
            for (j, _) in dots.iter().enumerate().take(i + 1) {
                WRITER
                    .lock()
                    .write_byte_at(row, col + j, dots[j], ColorCode::new(Color::Pink, Color::Black));
            }

            sleep_cycles(500_000_000);

            for j in 0..=i {
                WRITER
                    .lock()
                    .write_byte_at(row, col + j, b' ', ColorCode::new(Color::Pink, Color::Black));
            }
        }
    }
}

pub trait Testable {
    fn run(&self);
}

impl<T: Fn()> Testable for T {
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}

pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}

pub fn test_panic_handler(info: &core::panic::PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    hlt_loop();
}

#[cfg(test)]
use bootloader_api::{BootInfo, entry_point};
pub use kernel_lib::{serial_print, serial_println};

#[cfg(test)]
entry_point!(test_kernel_main, config = &BOOTLOADER_CONFIG);

#[cfg(test)]
fn test_kernel_main(_boot_info: &'static mut BootInfo) -> ! {
    init();
    test_main();
    hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    test_panic_handler(info)
}
