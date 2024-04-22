mod buffer;
mod colors;
pub mod console;
mod writer;

use colors::{Color, ColorCode};
use writer::Writer;
use core::fmt;
use buffer::Buffer;
use lazy_static::lazy_static;
use crate::custom_types::spin_lock::SpinLock;

// писатель
lazy_static! {
    pub static ref WRITER: SpinLock<Writer> = SpinLock::new(
        Writer::new(0,
        ColorCode::new(Color::LightBlue, Color::Black),
        unsafe { &mut *(0xb8000 as *mut Buffer) })
    );
}

//ниже идёт реализация макросов для вывода сообщений в консоль
#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    // отключение прерываний, пока объект заблокирован
    interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).expect("Printing failed");
    });
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}


pub fn start_message() {
    println!(r"/\/\/\/\/\/\/\/\/\/\/\//\/\/\/\/\//\/\/\/\/\/\/\/\/\/\/\/\/\//\/\/\/\/\//\/\/\/\");
    println!("\t\t\t___________________________________");
    println!("\t\t\t|    ____             _           |");
    println!("\t\t\t|    |  _ \\ _   _ ___| |_         |");
    println!("\t\t\t|    | |_) | | | / __| __|        |");
    println!("\t\t\t|    |  _ <| |_| \\__ \\ |_         |");
    println!("\t\t\t|    |_| \\_\\\\__,_|___/\\__|        |");
    println!("\t\t\t|_________________________________|");
    println!(r"/\/\/\/\/\/\/\/\/\/\/\//\/\/\/\/\//\/\/\/\/\/\/\/\/\/\/\/\/\//\/\/\/\/\//\/\/\/\");
}

//тесты для буфера
// #[test_case]
// fn test_println_output() {
//     use core::fmt::Write;
//     use x86_64::instructions::interrupts;
//     use crate::vga_buffer::buffer::BUFFER_HEIGHT;
//
//     let s = "Some test string that fits on a single line";
//     interrupts::without_interrupts(|| {
//         let mut writer = WRITER.lock();
//         writeln!(writer, "\n{}", s).expect("writeln failed");
//         for (i, c) in s.chars().enumerate() {
//             let screen_char = writer.buffer.chars[BUFFER_HEIGHT - 2][i].read();
//             assert_eq!(char::from(screen_char.ascii_character), c);
//         }
//     });
// }