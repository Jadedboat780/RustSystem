#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rust_system::test_runner)]
#![reexport_test_harness_main = "test_main"]

use bootloader_api::{BootInfo, entry_point};
use kernel_lib::console::{COLUMNS, ColorCode, ROWS, Writer};
use rust_system::println;

entry_point!(main);

fn main(_boot_info: &'static mut BootInfo) -> ! {
    test_main();
    rust_system::hlt_loop()
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    rust_system::test_panic_handler(info)
}

#[test_case]
fn test_println_many() {
    for num in 0..=100 {
        println!("Output the number {}", num);
    }
}

#[test_case]
fn test_writer_output() {
    use core::fmt::Write;

    let s = "Some test string that fits on a single line";
    assert!(s.len() <= COLUMNS);

    let mut writer = Writer::new(ColorCode::DEFAULT);
    writeln!(writer, "\n{}", s).expect("writeln failed");
    for (i, c) in s.chars().enumerate() {
        let screen_char = writer.char_at(ROWS - 2, i).expect("cell out of bounds");
        assert_eq!(char::from(screen_char.ascii_character), c);
    }
}
