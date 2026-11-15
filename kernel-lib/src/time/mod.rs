mod date_time;
mod instant;
pub mod rtc;

pub use date_time::{DateTime, DateDisplay, TimeDisplay, TimeError, Weekday};
pub use instant::Instant;

use crate::pit;
use core::{
    sync::atomic::{AtomicU64, Ordering::Relaxed},
    time::Duration,
};
use x86_64::instructions::{hlt, interrupts};

const NANOS_PER_SEC: u128 = 1_000_000_000;
const FALLBACK_TIME: DateTime = match DateTime::new(2025, 1, 1, 12, 0, 0) {
    Ok(time) => time,
    Err(_) => DateTime::UNIX_EPOCH,
};

static TICKS: AtomicU64 = AtomicU64::new(0);
static BOOT_UNIX: AtomicU64 = AtomicU64::new(0);

pub fn init() {
    set(rtc::read().unwrap_or(FALLBACK_TIME));
}

#[inline]
pub fn tick() -> u64 {
    TICKS.fetch_add(1, Relaxed) + 1
}

#[inline]
pub fn ticks() -> u64 {
    TICKS.load(Relaxed)
}

pub fn ticks_to_duration(ticks: u64) -> Duration {
    let nanos = ticks as u128 * pit::DIVISOR as u128 * NANOS_PER_SEC / pit::BASE_FREQUENCY as u128;
    Duration::from_nanos(nanos as u64)
}

pub fn duration_to_ticks(duration: Duration) -> u64 {
    let numerator = duration.as_nanos() * pit::BASE_FREQUENCY as u128;
    let denominator = pit::DIVISOR as u128 * NANOS_PER_SEC;
    numerator.div_ceil(denominator) as u64
}

pub fn uptime() -> Duration {
    ticks_to_duration(ticks())
}

pub fn now() -> DateTime {
    DateTime::from_unix(BOOT_UNIX.load(Relaxed) + uptime().as_secs())
}

pub fn set(time: DateTime) {
    BOOT_UNIX.store(time.to_unix().saturating_sub(uptime().as_secs()), Relaxed);
}

pub fn sleep(duration: Duration) {
    if interrupts::are_enabled() {
        let deadline = Instant::now() + duration;
        while Instant::now() < deadline {
            hlt();
        }
    } else {
        pit::wait_periods(duration_to_ticks(duration));
    }
}
