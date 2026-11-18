#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use bootloader_api::{BootInfo, entry_point};
use kernel_lib::{
    gdt::{self, DOUBLE_FAULT_IST_INDEX},
    sync::lazy::Lazy,
};
use rust_system::{QemuExitCode, exit_qemu, serial_print, serial_println};
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

entry_point!(main);

fn main(_boot_info: &'static mut BootInfo) -> ! {
    serial_print!("stack_overflow::stack_overflow...\t");

    gdt::init();
    init_test_idt();
    stack_overflow();

    panic!("Execution continued after stack overflow");
}

#[allow(unconditional_recursion)]
fn stack_overflow() {
    stack_overflow();
    unsafe { core::ptr::read_volatile(&0) };
}

static TEST_IDT: Lazy<InterruptDescriptorTable> = Lazy::new(|| {
    let mut idt = InterruptDescriptorTable::new();
    unsafe {
        idt.double_fault
            .set_handler_fn(test_double_fault_handler)
            .set_stack_index(DOUBLE_FAULT_IST_INDEX);
    }

    idt
});

pub fn init_test_idt() {
    TEST_IDT.load();
}

extern "x86-interrupt" fn test_double_fault_handler(_stack_frame: InterruptStackFrame, _error_code: u64) -> ! {
    serial_println!("[ok]");
    exit_qemu(QemuExitCode::Success);
    rust_system::hlt_loop()
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    rust_system::test_panic_handler(info)
}
