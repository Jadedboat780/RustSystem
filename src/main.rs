#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(oper_system::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use oper_system::{print, println};
use bootloader::{entry_point, BootInfo};

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use oper_system::allocator;
    use oper_system::memory::{self, BootInfoFrameAllocator};
    use x86_64::{structures::paging::Page, VirtAddr};

    oper_system::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("Heap initialization failed");

    #[cfg(test)]
    test_main();

    print!(">>> ");
    oper_system::hlt_loop();
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("{}", info);
    oper_system::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    oper_system::test_panic_handler(info);
}