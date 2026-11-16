use x86_64::instructions::port::Port;

pub const BASE_FREQUENCY: u64 = 1_193_182;
pub const DIVISOR: u16 = 1193;

const CHANNEL_0_DATA: u16 = 0x40;
const COMMAND: u16 = 0x43;

const SELECT_CHANNEL_0: u8 = 0x00;
const ACCESS_LATCH_COUNT: u8 = 0x00;
const ACCESS_LOW_THEN_HIGH: u8 = 0x30;
const MODE_RATE_GENERATOR: u8 = 0x04;
const BINARY_COUNTER: u8 = 0x00;

pub fn init() {
    unsafe {
        Port::<u8>::new(COMMAND).write(SELECT_CHANNEL_0 | ACCESS_LOW_THEN_HIGH | MODE_RATE_GENERATOR | BINARY_COUNTER);
        let mut data = Port::<u8>::new(CHANNEL_0_DATA);
        let [low, high] = DIVISOR.to_le_bytes();
        data.write(low);
        data.write(high);
    }
}

fn read_counter() -> u16 {
    unsafe {
        Port::<u8>::new(COMMAND).write(SELECT_CHANNEL_0 | ACCESS_LATCH_COUNT);
        let mut data = Port::<u8>::new(CHANNEL_0_DATA);
        let low = data.read();
        let high = data.read();
        u16::from_le_bytes([low, high])
    }
}

pub fn wait_periods(periods: u64) {
    let mut elapsed = 0;
    let mut last = read_counter();
    while elapsed < periods {
        let current = read_counter();
        if current > last {
            elapsed += 1;
        }
        last = current;
        core::hint::spin_loop();
    }
}
