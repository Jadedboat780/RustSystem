// представление цветов в системе
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

// структура для определения цвета текста и фона
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ColorCode(u8);

impl ColorCode {
    // инициализируем цвета для текста и фона
    pub const fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }

    // устанавливает новый цвет фона
    pub fn set_background(&mut self, background: Color) {
        self.0 |= (background as u8) << 4;
    }

    // устанавливает новый цвет текста
    pub fn set_foreground(&mut self, foreground: Color) {
        self.0 = (foreground as u8) | (self.0 & 0xF0);
    }
}
