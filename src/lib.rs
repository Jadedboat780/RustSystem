#![no_std]
#![cfg_attr(test, no_main)]
#![feature(abi_x86_interrupt)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

pub mod allocator;
pub mod commands;
mod datetime;
pub mod gdt;
pub mod interrupts;
pub mod memory;
pub mod serial;
pub mod task;

use custom_types::spin_lock::SpinLock;
use lazy_static::lazy_static;
use vga::{
    buffer::Buffer,
    colors::{Color, ColorCode},
    writer::Writer,
};

pub fn init() {
    gdt::init(); // включение двойных ошибок цп
    interrupts::init_idt(); // инициализация исключений ЦП
    unsafe { interrupts::PICS.lock().initialize() }; // инициализации PIC
    x86_64::instructions::interrupts::enable(); // включение внешних прерываний
}

pub trait Testable {
    fn run(&self);
}

impl<T: Fn()> Testable for T {
    // запуск тестов (функции помеченные #[cfg(test)])
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

// обработка паники для тестов
pub fn test_panic_handler(info: &core::panic::PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    hlt_loop();
}

// перечисление кодов завершения для QEMU
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

// отправка информации в порт о коде завершения
pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

// позволяет процессору перейти в состояние сна в ожидании следующего прерывания
pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

lazy_static! {
    pub static ref WRITER: SpinLock<Writer> = SpinLock::new(Writer::new(
        0,
        ColorCode::new(Color::Pink, Color::Black),
        unsafe { &mut *(0xb8000 as *mut Buffer) }
    ));
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

#[cfg(test)]
use bootloader::{BootInfo, entry_point};

#[cfg(test)]
entry_point!(test_kernel_main);

#[cfg(test)]
fn test_kernel_main(_boot_info: &'static BootInfo) -> ! {
    init();
    test_main();
    hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    test_panic_handler(info)
}
