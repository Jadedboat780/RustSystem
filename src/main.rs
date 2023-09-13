#![no_std]
#![no_main]

mod vga_buffer;

#[no_mangle]
fn _start() ->! {
    vga_buffer::print_something(b'H', "ello world");
    loop {}
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}