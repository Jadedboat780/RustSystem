use super::{
    buffer::{BUFFER_HEIGHT, BUFFER_WIDTH, Buffer, ScreenChar},
    colors::{Color, ColorCode},
};
use core::fmt;

pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    pub fn new(column_position: usize, color_code: ColorCode, buffer: &'static mut Buffer) -> Self {
        Self {
            column_position,
            color_code,
            buffer,
        }
    }

    pub fn set_color_code(&mut self, new_code: Color) {
        self.color_code.set_foreground(new_code)
    }

    pub fn set_column_position(&mut self, new_position: usize) {
        self.column_position = new_position;
    }

    pub fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    fn indentation(&mut self) {
        self.column_position += 4;
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }

    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            b'\t' => self.indentation(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }
                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code: self.color_code,
                });
                self.column_position += 1;
            }
        }
    }

    pub fn write_byte_at(&mut self, row: usize, col: usize, byte: u8, color_code: ColorCode) {
        if row < BUFFER_HEIGHT && col < BUFFER_WIDTH {
            self.buffer.chars[row][col].write(ScreenChar {
                ascii_character: byte,
                color_code,
            });
        }
    }

    pub fn write_string_at(&mut self, row: usize, col: usize, s: &str, color_code: ColorCode) {
        for (i, byte) in s.bytes().enumerate() {
            self.write_byte_at(row, col + i, byte, color_code);
        }
    }

    pub fn write_string(&mut self, s: &str) {
        s.bytes().for_each(|byte| match byte {
            0x20..=0x7e | b'\n' | b'\t' => self.write_byte(byte),
            _ => self.write_byte(0xfe),
        });
    }

    pub fn delete_char(&mut self) {
        if self.column_position >= 22 {
            self.column_position -= 1;

            let row = BUFFER_HEIGHT - 1;
            let col = self.column_position;

            self.buffer.chars[row][col].write(ScreenChar {
                ascii_character: b' ',
                color_code: self.color_code,
            });
        }
    }

    pub fn clear_screen(&mut self) {
        for row in 0..BUFFER_HEIGHT {
            self.clear_row(row);
        }
        self.column_position = 0;
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

// #[test_case]
// fn test_println_output() {
//     use core::fmt::Write;
//     use x86_64::instructions::interrupts;
//     use crate::buffer::BUFFER_HEIGHT;
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
//
