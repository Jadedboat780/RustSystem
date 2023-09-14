#![no_std]
#![no_main]

mod vga_buffer;

#[no_mangle]
fn _start() ->! {
    println!("Hello\nWorld{}", 23);
    loop {}
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}