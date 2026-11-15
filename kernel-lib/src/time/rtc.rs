use super::{DateTime, TimeError};
use x86_64::instructions::{interrupts, port::Port};

const ADDRESS: u16 = 0x70;
const DATA: u16 = 0x71;

const SECONDS: u8 = 0x00;
const MINUTES: u8 = 0x02;
const HOURS: u8 = 0x04;
const DAY: u8 = 0x07;
const MONTH: u8 = 0x08;
const YEAR: u8 = 0x09;
const STATUS_A: u8 = 0x0A;
const STATUS_B: u8 = 0x0B;

const UPDATE_IN_PROGRESS: u8 = 0x80;
const BINARY_MODE: u8 = 0x04;
const HOUR_24_MODE: u8 = 0x02;
const PM: u8 = 0x80;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RawRtc {
    pub seconds: u8,
    pub minutes: u8,
    pub hours: u8,
    pub day: u8,
    pub month: u8,
    pub year: u8,
}

pub fn read() -> Result<DateTime, TimeError> {
    let (raw, status_b) = interrupts::without_interrupts(|| {
        let mut last = read_raw();
        loop {
            let current = read_raw();
            if current == last {
                break (current, read_register(STATUS_B));
            }
            last = current;
        }
    });
    decode(raw, status_b)
}

pub fn decode(raw: RawRtc, status_b: u8) -> Result<DateTime, TimeError> {
    let convert = |value: u8| {
        if status_b & BINARY_MODE != 0 {
            value
        } else {
            bcd_to_binary(value)
        }
    };

    let is_pm = raw.hours & PM != 0;
    let mut hours = convert(raw.hours & !PM);
    if status_b & HOUR_24_MODE == 0 {
        hours = hours % 12 + if is_pm { 12 } else { 0 };
    }

    DateTime::new(
        2000 + convert(raw.year) as u16,
        convert(raw.month),
        convert(raw.day),
        hours,
        convert(raw.minutes),
        convert(raw.seconds),
    )
}

pub const fn bcd_to_binary(value: u8) -> u8 {
    (value >> 4) * 10 + (value & 0x0F)
}

fn read_raw() -> RawRtc {
    while read_register(STATUS_A) & UPDATE_IN_PROGRESS != 0 {
        core::hint::spin_loop();
    }
    RawRtc {
        seconds: read_register(SECONDS),
        minutes: read_register(MINUTES),
        hours: read_register(HOURS),
        day: read_register(DAY),
        month: read_register(MONTH),
        year: read_register(YEAR),
    }
}

fn read_register(register: u8) -> u8 {
    unsafe {
        Port::<u8>::new(ADDRESS).write(register);
        Port::<u8>::new(DATA).read()
    }
}
