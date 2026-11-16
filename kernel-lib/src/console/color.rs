#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Orange = 6,
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

impl Color {
    const ALL: [Color; 16] = [
        Color::Black,
        Color::Blue,
        Color::Green,
        Color::Cyan,
        Color::Red,
        Color::Magenta,
        Color::Orange,
        Color::LightGray,
        Color::DarkGray,
        Color::LightBlue,
        Color::LightGreen,
        Color::LightCyan,
        Color::LightRed,
        Color::Pink,
        Color::Yellow,
        Color::White,
    ];

    pub const fn from_u4(value: u8) -> Color {
        Self::ALL[(value & 0x0F) as usize]
    }

    pub const fn rgb(self) -> (u8, u8, u8) {
        match self {
            Color::Black => (0x00, 0x00, 0x00),
            Color::Blue => (0x00, 0x00, 0xAA),
            Color::Green => (0x00, 0xAA, 0x00),
            Color::Cyan => (0x00, 0xAA, 0xAA),
            Color::Red => (0xAA, 0x00, 0x00),
            Color::Magenta => (0xAA, 0x00, 0xAA),
            Color::Orange => (0xFF, 0x8C, 0x00),
            Color::LightGray => (0xAA, 0xAA, 0xAA),
            Color::DarkGray => (0x55, 0x55, 0x55),
            Color::LightBlue => (0x55, 0x55, 0xFF),
            Color::LightGreen => (0x55, 0xFF, 0x55),
            Color::LightCyan => (0x55, 0xFF, 0xFF),
            Color::LightRed => (0xFF, 0x55, 0x55),
            Color::Pink => (0xFF, 0x55, 0xFF),
            Color::Yellow => (0xFF, 0xFF, 0x55),
            Color::White => (0xFF, 0xFF, 0xFF),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ColorCode(u8);

impl ColorCode {
    pub const DEFAULT: ColorCode = ColorCode::new(Color::Orange, Color::Black);

    pub const fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }

    pub const fn foreground(self) -> Color {
        Color::from_u4(self.0)
    }

    pub const fn background(self) -> Color {
        Color::from_u4(self.0 >> 4)
    }

    pub const fn with_foreground(self, foreground: Color) -> ColorCode {
        ColorCode((self.0 & 0xF0) | foreground as u8)
    }

    pub const fn with_background(self, background: Color) -> ColorCode {
        ColorCode((self.0 & 0x0F) | (background as u8) << 4)
    }
}

impl Default for ColorCode {
    fn default() -> Self {
        Self::DEFAULT
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
