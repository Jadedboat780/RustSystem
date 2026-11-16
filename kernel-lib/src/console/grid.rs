use super::color::ColorCode;

pub const COLUMNS: usize = 80;
pub const ROWS: usize = 25;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScreenChar {
    pub ascii_character: u8,
    pub color_code: ColorCode,
}

impl ScreenChar {
    pub const fn blank(color_code: ColorCode) -> Self {
        Self {
            ascii_character: b' ',
            color_code,
        }
    }
}

pub(crate) struct Grid {
    cells: [[ScreenChar; COLUMNS]; ROWS],
}

impl Grid {
    pub const fn new(color_code: ColorCode) -> Self {
        Self {
            cells: [[ScreenChar::blank(color_code); COLUMNS]; ROWS],
        }
    }

    #[inline]
    pub fn get(&self, row: usize, col: usize) -> ScreenChar {
        self.cells[row][col]
    }

    #[inline]
    pub fn set(&mut self, row: usize, col: usize, ch: ScreenChar) -> bool {
        let cell = &mut self.cells[row][col];
        if *cell == ch {
            return false;
        }
        *cell = ch;
        true
    }
}
