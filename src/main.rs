#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rust_system::test_runner)]
#![reexport_test_harness_main = "test_main"]

use bootloader_api::{BootInfo, entry_point};
use rust_system::{BOOTLOADER_CONFIG, Kernel};

entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    let kernel = Kernel::init(boot_info);
    log::info!("RustSystem {} started", env!("CARGO_PKG_VERSION"));

    #[cfg(test)]
    test_main();

    #[cfg(not(test))]
    {
        rust_system::console::set_serial_mirror(true);
        rust_system::commands::show_splash();
    }

    kernel.run()
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    rust_system::console::print_panic(info);
    rust_system::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    rust_system::test_panic_handler(info);
}
