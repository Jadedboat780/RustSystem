#![no_std]
extern crate alloc;

mod date_time;
use core::sync::atomic::AtomicUsize;
pub use date_time::*;


pub static TICKS: AtomicUsize = AtomicUsize::new(0);

#[inline(always)]
fn rdtsc() -> u64 {
    let hi: u32;
    let lo: u32;
    unsafe {
        core::arch::asm!(
        "rdtsc",
        out("edx") hi,
        out("eax") lo,
        options(nomem, nostack)
        );
    }
    ((hi as u64) << 32) | (lo as u64)
}

pub fn sleep_cycles(cycles: u64) {
    let start = rdtsc() ;
    while rdtsc() - start < cycles {}
}