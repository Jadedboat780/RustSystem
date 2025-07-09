use crate::{WRITER, print, commands::Command};
use alloc::string::String;
use core::ops::DerefMut;
use custom_types::spin_lock::SpinLock;

const ENTER: u8 = 0x1C;
const BACKSPACE: u8 = 0x0E;
static LINE_BUFFER: SpinLock<String> = SpinLock::new(String::new());

#[inline]
fn get_scancode(scancode: u8) -> Option<char> {
    match scancode {
        0x02 => Some('1'),
        0x03 => Some('2'),
        0x04 => Some('3'),
        0x05 => Some('4'),
        0x06 => Some('5'),
        0x07 => Some('6'),
        0x08 => Some('7'),
        0x09 => Some('8'),
        0x0A => Some('9'),
        0x0B => Some('0'),
        0x0C => Some('-'),
        0x0D => Some('='),
        BACKSPACE => Some('#'),
        0x0F => Some('\t'),
        0x10 => Some('q'),
        0x11 => Some('w'),
        0x12 => Some('e'),
        0x13 => Some('r'),
        0x14 => Some('t'),
        0x15 => Some('y'),
        0x16 => Some('u'),
        0x17 => Some('i'),
        0x18 => Some('o'),
        0x19 => Some('p'),
        0x1A => Some('['),
        0x1B => Some(']'),
        ENTER => Some('\n'),
        0x1E => Some('a'),
        0x1F => Some('s'),
        0x20 => Some('d'),
        0x21 => Some('f'),
        0x22 => Some('g'),
        0x23 => Some('h'),
        0x24 => Some('j'),
        0x25 => Some('k'),
        0x26 => Some('l'),
        0x27 => Some(';'),
        0x28 => Some('\''),
        0x2B => Some('\\'),
        0x2C => Some('z'),
        0x2D => Some('x'),
        0x2E => Some('c'),
        0x2F => Some('v'),
        0x30 => Some('b'),
        0x31 => Some('n'),
        0x32 => Some('m'),
        0x33 => Some(','),
        0x34 => Some('.'),
        0x35 => Some('/'),
        0x37 => Some('*'),
        0x39 => Some(' '),
        0x4B | 0x4D => Some('%'),
        0x50 => Some('|'),
        _ => None,
    }
}

#[inline]
pub fn print_scancode(scancode: u8) {
    match get_scancode(scancode) {
        Some('\n') => {
            let buffer = core::mem::take(LINE_BUFFER.lock().deref_mut());
            Command::from(buffer.as_str()).execute();
        }
        Some('#') => {
            LINE_BUFFER.lock().pop();
            WRITER.lock().delete_char();
        }
        Some(char) => {
            print!("{}", char);
            LINE_BUFFER.lock().push(char);
        }
        None => {}
    }
}
