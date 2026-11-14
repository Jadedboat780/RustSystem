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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ColorCode(u8);

impl ColorCode {
    pub const fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }

    pub const fn foreground(self) -> u8 {
        self.0 & 0x0F
    }

    pub const fn background(self) -> u8 {
        self.0 >> 4
    }

    pub fn set_background(&mut self, background: Color) {
        self.0 |= (background as u8) << 4;
    }

    pub fn set_foreground(&mut self, foreground: Color) {
        self.0 = (foreground as u8) | (self.0 & 0xF0);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidColorCode;

impl TryFrom<u64> for ColorCode {
    type Error = InvalidColorCode;

    fn try_from(code: u64) -> Result<Self, Self::Error> {
        u8::try_from(code).map(ColorCode).map_err(|_| InvalidColorCode)
    }
}
