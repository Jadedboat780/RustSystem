use crate::{commands::Command, console, print};
use alloc::string::String;
use core::{
    ops::DerefMut,
    sync::atomic::{AtomicBool, Ordering::Relaxed},
};
use kernel_lib::sync::IrqSpinLock;

const ENTER: u8 = 0x1C;
const BACKSPACE: u8 = 0x0E;
const LEFT_SHIFT: u8 = 0x2A;
const RIGHT_SHIFT: u8 = 0x36;
const CAPS_LOCK: u8 = 0x3A;
const RELEASE: u8 = 0x80;
const EXTENDED: u8 = 0xE0;
const KEYPAD_SLASH: u8 = 0x35;

static LINE_BUFFER: IrqSpinLock<String> = IrqSpinLock::new(String::new());
static LEFT_SHIFT_HELD: AtomicBool = AtomicBool::new(false);
static RIGHT_SHIFT_HELD: AtomicBool = AtomicBool::new(false);
static CAPS_LOCK_HELD: AtomicBool = AtomicBool::new(false);
static CAPS_LOCK_ON: AtomicBool = AtomicBool::new(false);
static WAITING_FOR_ENTER: AtomicBool = AtomicBool::new(false);
static LAST_WAS_CARRIAGE_RETURN: AtomicBool = AtomicBool::new(false);
static EXTENDED_PENDING: AtomicBool = AtomicBool::new(false);

enum Key {
    Char(char),
    Enter,
    Backspace,
}

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
        0x29 => Some('`'),
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
fn shifted(c: char) -> char {
    match c {
        '1' => '!',
        '2' => '@',
        '3' => '#',
        '4' => '$',
        '5' => '%',
        '6' => '^',
        '7' => '&',
        '8' => '*',
        '9' => '(',
        '0' => ')',
        '-' => '_',
        '=' => '+',
        '[' => '{',
        ']' => '}',
        ';' => ':',
        '\'' => '"',
        '`' => '~',
        '\\' => '|',
        ',' => '<',
        '.' => '>',
        '/' => '?',
        c => c,
    }
}

#[inline]
fn apply_modifiers(c: char) -> char {
    let shift = LEFT_SHIFT_HELD.load(Relaxed) || RIGHT_SHIFT_HELD.load(Relaxed);
    if c.is_ascii_lowercase() {
        if shift ^ CAPS_LOCK_ON.load(Relaxed) {
            c.to_ascii_uppercase()
        } else {
            c
        }
    } else if shift {
        shifted(c)
    } else {
        c
    }
}

#[inline]
pub fn print_scancode(scancode: u8) {
    if scancode == EXTENDED {
        EXTENDED_PENDING.store(true, Relaxed);
        return;
    }
    if EXTENDED_PENDING.swap(false, Relaxed) {
        handle_extended(scancode);
        return;
    }
    match scancode {
        LEFT_SHIFT => LEFT_SHIFT_HELD.store(true, Relaxed),
        RIGHT_SHIFT => RIGHT_SHIFT_HELD.store(true, Relaxed),
        s if s == LEFT_SHIFT | RELEASE => LEFT_SHIFT_HELD.store(false, Relaxed),
        s if s == RIGHT_SHIFT | RELEASE => RIGHT_SHIFT_HELD.store(false, Relaxed),
        CAPS_LOCK => {
            if !CAPS_LOCK_HELD.swap(true, Relaxed) {
                CAPS_LOCK_ON.fetch_xor(true, Relaxed);
            }
        }
        s if s == CAPS_LOCK | RELEASE => CAPS_LOCK_HELD.store(false, Relaxed),
        ENTER => handle_key(Key::Enter),
        BACKSPACE => handle_key(Key::Backspace),
        _ => {
            if let Some(c) = get_scancode(scancode).map(apply_modifiers) {
                handle_key(Key::Char(c));
            }
        }
    }
}

fn handle_extended(scancode: u8) {
    match scancode {
        ENTER => handle_key(Key::Enter),
        KEYPAD_SLASH => handle_key(Key::Char('/')),
        _ => {}
    }
}

pub fn handle_serial_byte(byte: u8) {
    let after_carriage_return = LAST_WAS_CARRIAGE_RETURN.swap(byte == b'\r', Relaxed);
    match byte {
        b'\r' => handle_key(Key::Enter),
        b'\n' if !after_carriage_return => handle_key(Key::Enter),
        0x08 | 0x7F => handle_key(Key::Backspace),
        0x20..=0x7E => handle_key(Key::Char(byte as char)),
        _ => {}
    }
}

fn handle_key(key: Key) {
    if WAITING_FOR_ENTER.load(Relaxed) {
        if let Key::Enter = key {
            WAITING_FOR_ENTER.store(false, Relaxed);
        }
        return;
    }
    match key {
        Key::Enter => {
            let buffer = core::mem::take(LINE_BUFFER.lock().deref_mut());
            Command::from(buffer.as_str()).execute();
        }
        Key::Backspace => {
            if LINE_BUFFER.lock().pop().is_some() {
                console::delete_char();
            }
        }
        Key::Char(c) => {
            print!("{}", c);
            LINE_BUFFER.lock().push(c);
        }
    }
}

pub fn wait_for_enter() {
    WAITING_FOR_ENTER.store(true, Relaxed);
    while WAITING_FOR_ENTER.load(Relaxed) {
        x86_64::instructions::hlt();
    }
}

#[test_case]
fn test_extended_scancodes() {
    LINE_BUFFER.lock().clear();
    for &scancode in &[0xE0, 0x2A, 0xE0, 0x4B, 0xE0, 0x1D, 0xE0, 0x9D, 0xE0, 0x35, 0x1E, 0xE0, 0xAA] {
        print_scancode(scancode);
    }
    assert_eq!(LINE_BUFFER.lock().as_str(), "/a");
    assert!(!LEFT_SHIFT_HELD.load(Relaxed));
    LINE_BUFFER.lock().clear();
}

#[test_case]
fn test_serial_line_editing() {
    LINE_BUFFER.lock().clear();
    for &byte in b"ab\x7fc\x08d" {
        handle_serial_byte(byte);
    }
    assert_eq!(LINE_BUFFER.lock().as_str(), "ad");

    handle_serial_byte(0x1B);
    handle_serial_byte(0xC3);
    assert_eq!(LINE_BUFFER.lock().as_str(), "ad");
    LINE_BUFFER.lock().clear();
}
