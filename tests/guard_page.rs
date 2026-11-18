#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use bootloader_api::{BootInfo, entry_point};
use kernel_lib::{gdt, sync::Lazy};
use rust_system::{QemuExitCode, exit_qemu, hlt_loop, serial_print, serial_println};
use x86_64::{
    registers::control::Cr2,
    structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode},
};

entry_point!(main, config = &rust_system::BOOTLOADER_CONFIG);

static TEST_IDT: Lazy<InterruptDescriptorTable> = Lazy::new(|| {
    let mut idt = InterruptDescriptorTable::new();
    idt.page_fault.set_handler_fn(page_fault_handler);
    idt
});

fn main(boot_info: &'static mut BootInfo) -> ! {
    serial_print!("guard_page::double_fault_stack_guard...\t");
    rust_system::Kernel::init(boot_info);
    x86_64::instructions::interrupts::disable();
    TEST_IDT.load();

    let guard = gdt::double_fault_guard_page().start_address();
    unsafe { core::ptr::read_volatile(guard.as_ptr::<u8>()) };

    serial_println!("[failed]\n\nError: the guard page is mapped\n");
    exit_qemu(QemuExitCode::Failed);
    hlt_loop();
}

extern "x86-interrupt" fn page_fault_handler(_stack_frame: InterruptStackFrame, _error_code: PageFaultErrorCode) {
    let guard = gdt::double_fault_guard_page().start_address();
    if Cr2::read().ok() == Some(guard) {
        serial_println!("[ok]");
        exit_qemu(QemuExitCode::Success);
    } else {
        serial_println!("[failed]\n\nError: unexpected page fault at {:?}\n", Cr2::read());
        exit_qemu(QemuExitCode::Failed);
    }
    hlt_loop();
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    rust_system::test_panic_handler(info)
}
