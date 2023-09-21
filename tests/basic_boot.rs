#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(oper_system::test_runner)]
#![reexport_test_harness_main = "test_main"]

use oper_system::println;

#[no_mangle]
fn _start() -> ! {
    test_main();
    loop {}
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    oper_system::test_panic_handler(info)
}


#[test_case]
fn test_println_many() {
    for num in 0..=100 {
        println!("Output the number {}", num);
    }
}