#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rust_system::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use alloc::format;
use bootloader_api::{BootInfo, entry_point};
use core::time::Duration;
use kernel_lib::time::{
    self, DateTime, Instant, TimeError, Weekday,
    rtc::{self, RawRtc},
};
use x86_64::instructions::interrupts;

entry_point!(main, config = &rust_system::BOOTLOADER_CONFIG);

fn main(boot_info: &'static mut BootInfo) -> ! {
    rust_system::Kernel::init(boot_info);
    test_main();
    rust_system::hlt_loop();
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    rust_system::test_panic_handler(info)
}

fn date(year: u16, month: u8, day: u8, hours: u8, minutes: u8, seconds: u8) -> DateTime {
    DateTime::new(year, month, day, hours, minutes, seconds).expect("valid date")
}

#[test_case]
fn test_validation() {
    assert!(DateTime::new(2024, 2, 29, 0, 0, 0).is_ok());
    assert!(DateTime::new(2000, 2, 29, 0, 0, 0).is_ok());
    assert_eq!(DateTime::new(2023, 2, 29, 0, 0, 0), Err(TimeError::InvalidDay));
    assert_eq!(DateTime::new(2100, 2, 29, 0, 0, 0), Err(TimeError::InvalidDay));
    assert_eq!(DateTime::new(2024, 4, 31, 0, 0, 0), Err(TimeError::InvalidDay));
    assert_eq!(DateTime::new(2024, 1, 0, 0, 0, 0), Err(TimeError::InvalidDay));
    assert_eq!(DateTime::new(2024, 13, 1, 0, 0, 0), Err(TimeError::InvalidMonth));
    assert_eq!(DateTime::new(2024, 0, 1, 0, 0, 0), Err(TimeError::InvalidMonth));
    assert_eq!(DateTime::new(2024, 1, 1, 24, 0, 0), Err(TimeError::InvalidTime));
    assert_eq!(DateTime::new(2024, 1, 1, 0, 60, 0), Err(TimeError::InvalidTime));
    assert_eq!(DateTime::new(1969, 12, 31, 0, 0, 0), Err(TimeError::YearOutOfRange));
}

#[test_case]
fn test_with_time_and_date() {
    let value = date(2024, 2, 29, 12, 34, 56);
    assert_eq!(value.with_time(1, 2, 3), Ok(date(2024, 2, 29, 1, 2, 3)));
    assert_eq!(value.with_date(2023, 2, 29), Err(TimeError::InvalidDay));
}

#[test_case]
fn test_unix_known_values() {
    assert_eq!(DateTime::UNIX_EPOCH.to_unix(), 0);
    assert_eq!(date(2000, 3, 1, 0, 0, 0).to_unix(), 951_868_800);
    assert_eq!(date(2024, 2, 29, 12, 34, 56).to_unix(), 1_709_210_096);
    assert_eq!(date(2038, 1, 19, 3, 14, 8).to_unix(), 2_147_483_648);

    assert_eq!(DateTime::from_unix(0), DateTime::UNIX_EPOCH);
    assert_eq!(DateTime::from_unix(1_709_210_096), date(2024, 2, 29, 12, 34, 56));
    assert_eq!(DateTime::from_unix(u64::MAX), DateTime::MAX);
}

#[test_case]
fn test_unix_round_trip() {
    let mut timestamp = 0;
    while timestamp < DateTime::MAX.to_unix() {
        assert_eq!(DateTime::from_unix(timestamp).to_unix(), timestamp);
        timestamp += 7_777_777;
    }
}

#[test_case]
fn test_rollover() {
    assert_eq!(
        DateTime::from_unix(date(2023, 12, 31, 23, 59, 59).to_unix() + 1),
        date(2024, 1, 1, 0, 0, 0)
    );
    assert_eq!(
        DateTime::from_unix(date(2024, 2, 28, 23, 59, 59).to_unix() + 1),
        date(2024, 2, 29, 0, 0, 0)
    );
    assert_eq!(
        DateTime::from_unix(date(2023, 2, 28, 23, 59, 59).to_unix() + 1),
        date(2023, 3, 1, 0, 0, 0)
    );
    assert_eq!(
        DateTime::from_unix(date(2024, 4, 30, 23, 59, 59).to_unix() + 1),
        date(2024, 5, 1, 0, 0, 0)
    );
}

#[test_case]
fn test_weekday() {
    assert_eq!(DateTime::UNIX_EPOCH.weekday(), Weekday::Thursday);
    assert_eq!(date(2000, 3, 1, 0, 0, 0).weekday(), Weekday::Wednesday);
    assert_eq!(date(2024, 2, 29, 0, 0, 0).weekday(), Weekday::Thursday);
    assert_eq!(date(2038, 1, 19, 0, 0, 0).weekday(), Weekday::Tuesday);
}

#[test_case]
fn test_display() {
    let value = date(2024, 2, 9, 7, 4, 5);
    assert_eq!(format!("{}", value), "09.02.2024 07:04:05");
    assert_eq!(format!("{}", value.date()), "09.02.2024");
    assert_eq!(format!("{}", value.time()), "07:04");
    assert_eq!(format!("{}", value.weekday()), "Friday");
}

#[test_case]
fn test_rtc_decode_bcd_24h() {
    let raw = RawRtc {
        seconds: 0x56,
        minutes: 0x34,
        hours: 0x23,
        day: 0x29,
        month: 0x02,
        year: 0x24,
    };
    assert_eq!(rtc::decode(raw, 0x02), Ok(date(2024, 2, 29, 23, 34, 56)));
}

#[test_case]
fn test_rtc_decode_binary_12h() {
    let raw = |hours| RawRtc {
        seconds: 56,
        minutes: 34,
        hours,
        day: 29,
        month: 2,
        year: 24,
    };
    assert_eq!(rtc::decode(raw(0x80 | 1), 0x04).map(|t| t.hours()), Ok(13));
    assert_eq!(rtc::decode(raw(0x80 | 12), 0x04).map(|t| t.hours()), Ok(12));
    assert_eq!(rtc::decode(raw(12), 0x04).map(|t| t.hours()), Ok(0));
    assert_eq!(rtc::decode(raw(11), 0x04).map(|t| t.hours()), Ok(11));
}

#[test_case]
fn test_rtc_decode_rejects_garbage() {
    let raw = RawRtc {
        seconds: 0,
        minutes: 0,
        hours: 0,
        day: 0x31,
        month: 0x13,
        year: 0x24,
    };
    assert_eq!(rtc::decode(raw, 0x02), Err(TimeError::InvalidMonth));
}

#[test_case]
fn test_bcd_to_binary() {
    assert_eq!(rtc::bcd_to_binary(0x00), 0);
    assert_eq!(rtc::bcd_to_binary(0x09), 9);
    assert_eq!(rtc::bcd_to_binary(0x10), 10);
    assert_eq!(rtc::bcd_to_binary(0x59), 59);
}

#[test_case]
fn test_rtc_reads_real_time() {
    let value = rtc::read().expect("RTC returned an invalid date");
    assert!(value.year() >= 2024);
}

#[test_case]
fn test_tick_conversion() {
    assert_eq!(time::ticks_to_duration(0), Duration::ZERO);
    assert_eq!(time::ticks_to_duration(1000).as_nanos(), 999_847_466);
    assert_eq!(time::duration_to_ticks(Duration::from_secs(1)), 1001);
    assert_eq!(time::duration_to_ticks(Duration::ZERO), 0);
}

#[test_case]
fn test_sleep_and_instant() {
    let start = Instant::now();
    time::sleep(Duration::from_millis(20));
    let elapsed = start.elapsed();
    assert!(elapsed >= Duration::from_millis(20));
    assert!(elapsed < Duration::from_millis(500));
    assert_eq!(Instant::now() - start, Instant::now().duration_since(start));
}

#[test_case]
fn test_sleep_without_interrupts_returns() {
    interrupts::without_interrupts(|| time::sleep(Duration::from_millis(5)));
    assert!(interrupts::are_enabled());
}

#[test_case]
fn test_uptime_advances() {
    let before = time::uptime();
    time::sleep(Duration::from_millis(5));
    assert!(time::uptime() > before);
}

#[test_case]
fn test_set_and_now() {
    let value = date(2030, 6, 15, 10, 20, 30);
    time::set(value);
    let now = time::now();
    assert!(now >= value);
    assert!(now.to_unix() - value.to_unix() <= 1);
}
