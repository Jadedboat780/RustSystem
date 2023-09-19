#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(oper_system::test_runner)]
#![reexport_test_harness_main = "test_main"]

use oper_system::println;

#[no_mangle]
fn _start() -> ! {
    println!("Hello World");

    oper_system::init();
    x86_64::instructions::interrupts::int3();

    #[cfg(test)]
    test_main();

    println!("End");
    loop {}
}

#[cfg(not(test))] // new attribute
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    oper_system::test_panic_handler(info);
}

