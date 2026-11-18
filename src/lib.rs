#![no_std]
#![cfg_attr(test, no_main)]
#![feature(abi_x86_interrupt)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

pub mod allocator;
pub mod commands;
pub mod console;
pub mod interrupts;
pub mod kernel;
pub mod keyboard;
pub mod logger;
pub mod qemu;
pub mod syscalls;
pub mod testing;

pub use console::WRITER;
pub use kernel::{BOOTLOADER_CONFIG, Kernel, hlt_loop};
pub use kernel_lib::{serial_print, serial_println};
pub use qemu::{QemuExitCode, exit_qemu};
pub use testing::{Testable, test_panic_handler, test_runner};

#[cfg(test)]
bootloader_api::entry_point!(test_kernel_main, config = &BOOTLOADER_CONFIG);

#[cfg(test)]
fn test_kernel_main(boot_info: &'static mut bootloader_api::BootInfo) -> ! {
    Kernel::init(boot_info);
    test_main();
    hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    test_panic_handler(info)
}
