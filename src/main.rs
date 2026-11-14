#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rust_system::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use bootloader_api::{BootInfo, entry_point};
use kernel_lib::{
    paging::{self, BootInfoFrameAllocator},
    time::DateTime,
};
use rust_system::{BOOTLOADER_CONFIG, WRITER, allocator::init_heap, print};
use x86_64::VirtAddr;

entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    if let Some(framebuffer) = boot_info.framebuffer.as_mut() {
        WRITER.lock().attach_framebuffer(framebuffer);
    }

    rust_system::print_logo(24, 35);
    rust_system::init();

    let phys_mem_offset = boot_info
        .physical_memory_offset
        .into_option()
        .expect("Bootloader did not map physical memory");
    let mut mapper = unsafe { paging::init(VirtAddr::new(phys_mem_offset)) };
    let mut frame_allocator = BootInfoFrameAllocator::init(&boot_info.memory_regions);

    init_heap(&mut mapper, &mut frame_allocator).expect("Heap initialization failed");

    #[cfg(test)]
    test_main();

    print!("{}$ ", DateTime::now());
    rust_system::hlt_loop();
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    print!("{}", info);
    rust_system::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    rust_system::test_panic_handler(info);
}
