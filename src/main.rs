#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(oper_system::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use bootloader::{entry_point, BootInfo};
use oper_system::{
    print, println,
    vga_buffer::start_message,
};

// определяем точку входа
entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use oper_system::allocator;
    use oper_system::memory;
    use x86_64::VirtAddr;

    oper_system::init();

    // инициализация маппера страниц и аллокатор кадров
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator =  memory::BootInfoFrameAllocator::init(&boot_info.memory_map);

    // инициализация кучи
    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("Heap initialization failed");

    #[cfg(test)]
    test_main();

    // вывод стартового сообщения
    start_message();
    print!("<<< ");

    oper_system::hlt_loop();
}

// обработчик паники для системы
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("{}", info);
    oper_system::hlt_loop();
}

// обработчик паники для тестов
#[cfg(test)]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    oper_system::test_panic_handler(info);
}
