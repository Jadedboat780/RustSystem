#![no_std]
#![no_main]

use bootloader_api::{BootInfo, entry_point};
use rust_system::{QemuExitCode, exit_qemu, serial_print, serial_println};

entry_point!(main);

fn main(_boot_info: &'static mut BootInfo) -> ! {
    should_fail();
    serial_println!("[test did not panic]");
    exit_qemu(QemuExitCode::Failed);
    rust_system::hlt_loop()
}

fn should_fail() {
    serial_print!("should_panic::should_fail...\t");
    assert_eq!(0, 1);
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    serial_println!("[ok]");
    exit_qemu(QemuExitCode::Success);
    rust_system::hlt_loop()
}
