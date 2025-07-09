#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rust_system::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use bootloader::{BootInfo, entry_point};
use datetime::DateTime;
use rust_system::{allocator::init_heap, print};
use x86_64::VirtAddr;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    rust_system::print_logo(24, 35);
    rust_system::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = memory::BootInfoFrameAllocator::init(&boot_info.memory_map);

    init_heap(&mut mapper, &mut frame_allocator).expect("Heap initialization failed");

    #[cfg(test)]
    test_main();

    print!("{}$ ", DateTime::now());
    rust_system::hlt_loop();
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    // print!("{}", info);
    rust_system::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    rust_system::test_panic_handler(info);
}
