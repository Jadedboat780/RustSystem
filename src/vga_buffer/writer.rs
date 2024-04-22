use core::fmt;
use super::buffer::{ScreenChar, Buffer, BUFFER_HEIGHT, BUFFER_WIDTH};
use super::colors::{ColorCode, Color};

// структура для отслеживание состояния консоли
struct Condition {
    is_quotes_open: bool,
}

//структура для представления Писателя
pub struct Writer {
    column_position: usize, // текущая позицию в столбце
    color_code: ColorCode, //
    buffer: &'static mut Buffer, //
    condition: Condition // состояние консоли
}

impl Writer {
    pub fn new(column_position: usize, color_code: ColorCode, buffer: &'static mut Buffer) -> Self {
        let condition = Condition { is_quotes_open: false };
        Self {
            column_position,
            color_code,
            buffer,
            condition,
        }
    }

    pub fn get_color_code(&self) -> ColorCode { self.color_code }
    pub fn get_quotes_condition(&self) -> bool { self.condition.is_quotes_open }
    pub fn set_color_code(&mut self, new_code: Color) { self.color_code.set_foreground(new_code) }

    pub fn set_quotes_condition(&mut self, new_condition: bool) {
        self.condition.is_quotes_open = new_condition;
        if new_condition {
            self.set_color_code(Color::Pink);
        } else {
            self.set_color_code(Color::LightBlue);
        }
    }

    // метод для переноса на новую строку
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

    // метод для табуляции
    fn indentation(&mut self) {
        self.column_position += 4;
    }

    // метод для очистки строки(перезаписывает все её символы пробелом)
    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }

    // метод для записи одного байта ASCII
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

    // метод для записи сразу нескольких символов
    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' | b'\t' => self.write_byte(byte),
                _ => self.write_byte(0xfe),
            }
        }
    }

    // функция для удаления символов
    pub fn delete_char(&mut self) {
        if self.column_position >= 5 {
            self.column_position -= 1;

            let row = BUFFER_HEIGHT - 1;
            let col = self.column_position;

            self.buffer.chars[row][col].write(ScreenChar {
                ascii_character: b' ',
                color_code: self.color_code,
            });
        }
    }

    pub fn delete_string(&mut self) {
        while self.column_position >= 5 {
            self.delete_char()
        }
    }

    // fn check_on_delete_special_char(){
    //
    // }
}

// функционал для форматированного вывода данных в виде строки
impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}
