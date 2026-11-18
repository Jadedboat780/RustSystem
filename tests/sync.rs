#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rust_system::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use alloc::format;
use bootloader_api::{BootInfo, entry_point};
use core::sync::atomic::{AtomicUsize, Ordering::Relaxed};
use kernel_lib::sync::{IrqSpinLock, Lazy, SpinLock, lazy::InitInProgress};
use x86_64::instructions::interrupts;

entry_point!(main, config = &rust_system::BOOTLOADER_CONFIG);

fn main(boot_info: &'static mut BootInfo) -> ! {
    rust_system::Kernel::init(boot_info);
    test_main();
    rust_system::hlt_loop()
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    rust_system::test_panic_handler(info)
}

#[test_case]
fn test_lock() {
    let array = SpinLock::new([1, 2, 3, 4, 5]);
    assert!(!array.is_locked());

    let _guard = array.lock();
    assert!(array.is_locked());
}

#[test_case]
fn test_try_lock() {
    let array = SpinLock::new([1, 2, 3, 4, 5]);
    let guard_1 = array.try_lock();
    assert!(guard_1.is_some());

    let guard_2 = array.try_lock();
    assert!(guard_2.is_none());
}

#[test_case]
fn test_unlock() {
    let array = SpinLock::new([1, 2, 3, 4, 5]);
    let guard = array.lock();
    assert!(array.is_locked());

    drop(guard);
    assert!(!array.is_locked());
}

#[test_case]
fn test_mutation() {
    let value = SpinLock::new(0);
    *value.lock() += 1;
    *value.lock() += 1;
    assert_eq!(*value.lock(), 2);
}

#[test_case]
fn test_force_unlock() {
    let value = SpinLock::new(0);
    core::mem::forget(value.lock());
    assert!(value.try_lock().is_none());

    unsafe { value.force_unlock() };
    assert!(value.try_lock().is_some());
}

#[test_case]
fn test_irq_lock_disables_interrupts() {
    let value = IrqSpinLock::new(0);
    assert!(interrupts::are_enabled());

    let guard = value.lock();
    assert!(!interrupts::are_enabled());

    drop(guard);
    assert!(interrupts::are_enabled());
}

#[test_case]
fn test_irq_lock_nested() {
    let outer = IrqSpinLock::new(0);
    let inner = IrqSpinLock::new(0);

    let outer_guard = outer.lock();
    let inner_guard = inner.lock();
    drop(inner_guard);
    assert!(!interrupts::are_enabled());

    drop(outer_guard);
    assert!(interrupts::are_enabled());
}

#[test_case]
fn test_irq_lock_keeps_interrupts_disabled() {
    let value = IrqSpinLock::new(0);
    interrupts::without_interrupts(|| {
        drop(value.lock());
        assert!(!interrupts::are_enabled());
    });
    assert!(interrupts::are_enabled());
}

#[test_case]
fn test_irq_try_lock_failure_restores_interrupts() {
    let value = IrqSpinLock::new(0);
    let guard = value.lock();
    assert!(value.try_lock().is_none());
    assert!(!interrupts::are_enabled());

    drop(guard);
    assert!(value.try_lock().is_some());
    assert!(interrupts::are_enabled());
}

struct DropCounter(&'static AtomicUsize);

impl Drop for DropCounter {
    fn drop(&mut self) {
        self.0.fetch_add(1, Relaxed);
    }
}

#[test_case]
fn test_lazy_initializes_once() {
    static CALLS: AtomicUsize = AtomicUsize::new(0);
    static VALUE: Lazy<u32> = Lazy::new(|| {
        CALLS.fetch_add(1, Relaxed);
        42
    });

    assert_eq!(Lazy::get(&VALUE), None);
    assert_eq!(*VALUE, 42);
    assert_eq!(*VALUE, 42);
    assert_eq!(Lazy::get(&VALUE), Some(&42));
    assert_eq!(CALLS.load(Relaxed), 1);
}

#[test_case]
fn test_lazy_force_mut() {
    let mut value = Lazy::new(|| 1);
    *value += 1;
    assert_eq!(*Lazy::force_mut(&mut value), 2);
    assert_eq!(Lazy::get_mut(&mut value), Some(&mut 2));
}

#[test_case]
fn test_lazy_into_inner() {
    let value = Lazy::<u32, _>::new(|| 5);
    let f = Lazy::into_inner(value).expect_err("value should not be initialized");
    assert_eq!(f(), 5);

    let value = Lazy::new(|| 5);
    Lazy::force(&value);
    assert_eq!(Lazy::into_inner(value).ok(), Some(5));
}

#[test_case]
fn test_lazy_drops_init_function_if_unused() {
    static DROPS: AtomicUsize = AtomicUsize::new(0);
    let counter = DropCounter(&DROPS);
    let value = Lazy::<usize, _>::new(move || {
        let counter = counter;
        counter.0.load(Relaxed)
    });

    drop(value);
    assert_eq!(DROPS.load(Relaxed), 1);
}

#[test_case]
fn test_lazy_drops_value_once() {
    static DROPS: AtomicUsize = AtomicUsize::new(0);
    let value = Lazy::new(|| DropCounter(&DROPS));

    Lazy::force(&value);
    assert_eq!(DROPS.load(Relaxed), 0);

    drop(value);
    assert_eq!(DROPS.load(Relaxed), 1);
}

#[test_case]
fn test_lazy_try_force_detects_reentrancy() {
    static VALUE: Lazy<u32> = Lazy::new(|| {
        assert_eq!(Lazy::try_force(&VALUE), Err(InitInProgress));
        7
    });

    assert_eq!(Lazy::try_force(&VALUE), Ok(&7));
}

#[test_case]
fn test_lazy_init_runs_without_interrupts() {
    let value = Lazy::new(interrupts::are_enabled);
    assert!(interrupts::are_enabled());
    assert!(!*value);
    assert!(interrupts::are_enabled());
}

#[test_case]
fn test_lazy_debug_and_default() {
    let value: Lazy<u32> = Lazy::default();
    assert_eq!(format!("{:?}", value), "Lazy(<uninit>)");

    Lazy::force(&value);
    assert_eq!(format!("{:?}", value), "Lazy(0)");
}
