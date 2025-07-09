#![no_std]

use x86_64::instructions::port::Port;

pub fn init() {
    let frequency: u16 = 1193; // The frequency of the timer ~1мс (1193182 / 1000)

    unsafe {
        let mut command_port = Port::new(0x43);
        command_port.write(0x34u8); // PIT management: channel 0, mode 2

        let mut data_port = Port::new(0x40);
        data_port.write((frequency & 0xFF) as u8); // Junior byte
        data_port.write((frequency >> 8) as u8); // Senior byte
    }
}
