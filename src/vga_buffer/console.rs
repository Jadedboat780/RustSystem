use super::{WRITER, colors::Color};
use crate::custom_types::spin_lock::SpinLock;
use crate::{print, println};
use alloc::string::String;
use alloc::vec::Vec;
use core::mem;

static BUFFER_LINE: SpinLock<String> = SpinLock::new(String::new());
static BUFFER_HISTORY: SpinLock<Vec<String>> = SpinLock::new(Vec::new());
// static mut COUNTER_ARROW_CLICK: usize = 1;

fn command() {
    println!();
    WRITER.lock().set_color_code(Color::LightRed);

    match BUFFER_LINE.lock().as_str() {
        "hello" => {
            println!("                **    **  ********  **        **            **");
            println!("               **    **  ********  **        **         **     **");
            println!("              **    **  **        **        **         **      **");
            println!("             ********  ********  **        **         **      **");
            println!("            **    **  **        **        **         **      **");
            println!("           **    **  ********  ********  ********    **    **");
            println!("          **    **  ********  ********  ********       **\n");
        }
        "help" => println!(
            "\tThis is OS write on Rust lang\n\tYou are using version {}\n\tAt this point in time, this system does not know how to do anything",
            env!("CARGO_PKG_VERSION")
        ),
        "version" => println!("\tActual version: {}", env!("CARGO_PKG_VERSION")),
        "" => (),
        command => println!("Command not found: {}", command),
    }

    WRITER.lock().set_color_code(Color::LightBlue);
    let str1 = mem::take(&mut *BUFFER_LINE.lock()); //очищаем буффер и получаем из него значение
    BUFFER_HISTORY.lock().push(str1);
    print!("<<< ");
}

#[inline]
pub fn keyboard(scancode: u8) {
    let lower_char = |scancode: u8| -> Option<char> {
        match scancode {
            0x01 => panic!("GOODBYE"), //escape
            0x02 => Some('1'),
            0x03 => Some('2'),
            0x04 => Some('3'),
            0x05 => Some('4'),
            0x06 => Some('5'),
            0x07 => Some('6'),
            0x08 => Some('7'),
            0x09 => Some('8'),
            0x0a => Some('9'),
            0x0b => Some('0'),
            0x0C => Some('-'),
            0x0D => Some('='),
            0x0E => Some('#'),
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
            0x1C => Some('\n'),
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
            // 0x2A =>
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
            0x48 => {
                // arrow_click();
                None
            } //up
            0x4B => Some('%'), //left
            0x4D => Some('%'), //right
            0x50 => Some('|'), //down
            _ => None,
        }
    };

    match lower_char(scancode) {
        Some('\n') => {
            command();
        }
        Some('#') => {
            WRITER.lock().delete_char();
            let del_chat = BUFFER_LINE.lock().pop().unwrap();
            if del_chat == '\'' {
                // WRITER.lock().set_color_code()
            }
        }
        Some('-') => {
            WRITER.lock().set_color_code(Color::Green);
            print!("-");
            BUFFER_LINE.lock().push('-')
        }
        Some('\'') => quotes(),
        Some(char) => {
            print!("{}", char);
            BUFFER_LINE.lock().push(char)
        }
        None => (),
    }
}

fn quotes() {
    if WRITER.lock().get_quotes_condition() {
        print!("'");
        WRITER.lock().set_quotes_condition(false);
    } else {
        WRITER.lock().set_quotes_condition(true);
        print!("'");
    }
}

// fn arrow_up_click() {
//     let number_element = unsafe { COUNTER_ARROW_CLICK };
//     let length = BUFFER_HISTORY.lock().len();
//
//     if number_element > length{
//         unsafe {COUNTER_ARROW_CLICK = 1}
//         return;
//     }
//     WRITER.lock().delete_string();
//
//     let binding = BUFFER_HISTORY.lock();
//     let s = binding[length-number_element].as_str();
//     print!("{}", s);
//     unsafe {COUNTER_ARROW_CLICK += 1}
// }
//
// fn arrow_down_click(){
//     todo!()
// }

// let upper_char = |scancode: u8| -> Option<char>{
// match scancode {
// 0x01 => panic!("GOODBYE"), //escape
// 0x02 => Some('!'),
// 0x03 => Some('@'),
// 0x04 => Some('#'),
// 0x05 => Some('$'),
// 0x06 => Some('%'),
// 0x07 => Some('^'),
// 0x08 => Some('&'),
// 0x09 => Some('*'),
// 0x0a => Some('('),
// 0x0b => Some(')'),
// 0x0C => Some('_'),
// 0x0D => Some('+'),
// 0x0E => Some('#'), //Ё
// 0x10 => Some('Q'),
// 0x11 => Some('W'),
// 0x12 => Some('E'),
// 0x13 => Some('R'),
// 0x14 => Some('T'),
// 0x15 => Some('Y'),
// 0x16 => Some('U'),
// 0x17 => Some('I'),
// 0x18 => Some('O'),
// 0x19 => Some('P'),
// 0x1A => Some('{'),
// 0x1B => Some('}'),
// 0x1C => Some('\n'),
// 0x1E => Some('A'),
// 0x1F => Some('S'),
// 0x20 => Some('D'),
// 0x21 => Some('F'),
// 0x22 => Some('G'),
// 0x23 => Some('H'),
// 0x24 => Some('J'),
// 0x25 => Some('K'),
// 0x26 => Some('L'),
// 0x27 => Some(':'),
// 0x28 => Some('"'),
// 0x2B => Some('|'),
// 0x2C => Some('Z'),
// 0x2D => Some('X'),
// 0x2E => Some('C'),
// 0x2F => Some('V'),
// 0x30 => Some('B'),
// 0x31 => Some('N'),
// 0x32 => Some('M'),
// 0x33 => Some('<'),
// 0x34 => Some('>'),
// 0x35 => Some('?'),
// 0x37 => Some('*'),
// _ => None,
// }
// };
