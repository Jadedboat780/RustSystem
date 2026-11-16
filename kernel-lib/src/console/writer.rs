use super::{
    color::{Color, ColorCode},
    grid::{COLUMNS, Grid, ROWS, ScreenChar},
    render::Renderer,
};
use bootloader_api::info::FrameBuffer;
use core::fmt;

const TAB_WIDTH: usize = 4;
const REPLACEMENT: u8 = 0xFE;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OutOfBounds;

impl fmt::Display for OutOfBounds {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("position is outside the screen")
    }
}

pub struct Writer {
    grid: Grid,
    renderer: Option<Renderer>,
    row: usize,
    col: usize,
    color: ColorCode,
}

impl Writer {
    pub const fn new(color: ColorCode) -> Self {
        Self {
            grid: Grid::new(color),
            renderer: None,
            row: ROWS - 1,
            col: 0,
            color,
        }
    }

    pub fn attach_framebuffer(&mut self, framebuffer: &'static mut FrameBuffer) {
        let mut renderer = Renderer::new(framebuffer);
        renderer.clear();
        for row in 0..ROWS {
            for col in 0..COLUMNS {
                let ch = self.grid.get(row, col);
                if ch.ascii_character != b' ' || ch.color_code.background() != Color::Black {
                    renderer.draw(row, col, ch);
                }
            }
        }
        self.renderer = Some(renderer);
    }

    pub const fn width(&self) -> usize {
        COLUMNS
    }

    pub const fn height(&self) -> usize {
        ROWS
    }

    pub const fn color(&self) -> ColorCode {
        self.color
    }

    pub fn set_color(&mut self, color: ColorCode) {
        self.color = color;
    }

    pub fn set_foreground(&mut self, foreground: Color) {
        self.color = self.color.with_foreground(foreground);
    }

    pub fn set_background(&mut self, background: Color) {
        self.color = self.color.with_background(background);
    }

    pub const fn cursor(&self) -> (usize, usize) {
        (self.row, self.col)
    }

    pub fn set_cursor(&mut self, row: usize, col: usize) -> Result<(), OutOfBounds> {
        check(row, col)?;
        self.row = row;
        self.col = col;
        Ok(())
    }

    pub fn char_at(&self, row: usize, col: usize) -> Option<ScreenChar> {
        check(row, col).ok().map(|_| self.grid.get(row, col))
    }

    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            b'\t' => self.tab(),
            byte => {
                if self.col >= COLUMNS {
                    self.new_line();
                }
                self.put(self.row, self.col, byte, self.color);
                self.col += 1;
            }
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7E | b'\n' | b'\t' => self.write_byte(byte),
                _ => self.write_byte(REPLACEMENT),
            }
        }
    }

    pub fn write_byte_at(&mut self, row: usize, col: usize, byte: u8, color: ColorCode) -> Result<(), OutOfBounds> {
        check(row, col)?;
        self.put(row, col, byte, color);
        Ok(())
    }

    pub fn write_string_at(&mut self, row: usize, col: usize, s: &str, color: ColorCode) -> Result<usize, OutOfBounds> {
        check(row, col)?;
        let written = s.len().min(COLUMNS - col);
        for (i, byte) in s.bytes().take(written).enumerate() {
            self.put(row, col + i, byte, color);
        }
        Ok(written)
    }

    pub fn new_line(&mut self) {
        self.col = 0;
        if self.row + 1 < ROWS {
            self.row += 1;
        } else {
            self.scroll_up();
            self.clear_row(ROWS - 1);
        }
    }

    pub fn delete_char(&mut self) {
        if self.col > 0 {
            self.col -= 1;
            self.put(self.row, self.col, b' ', self.color);
        }
    }

    pub fn clear_line(&mut self) {
        self.clear_row(self.row);
        self.col = 0;
    }

    pub fn clear_screen(&mut self) {
        for row in 0..ROWS {
            self.clear_row(row);
        }
        self.row = 0;
        self.col = 0;
    }

    fn tab(&mut self) {
        let next = (self.col / TAB_WIDTH + 1) * TAB_WIDTH;
        if next >= COLUMNS {
            self.new_line();
            return;
        }
        while self.col < next {
            self.put(self.row, self.col, b' ', self.color);
            self.col += 1;
        }
    }

    fn clear_row(&mut self, row: usize) {
        for col in 0..COLUMNS {
            self.put(row, col, b' ', self.color);
        }
    }

    fn scroll_up(&mut self) {
        for row in 0..ROWS - 1 {
            for col in 0..COLUMNS {
                self.put_char(row, col, self.grid.get(row + 1, col));
            }
        }
    }

    fn put(&mut self, row: usize, col: usize, byte: u8, color: ColorCode) {
        self.put_char(
            row,
            col,
            ScreenChar {
                ascii_character: byte,
                color_code: color,
            },
        );
    }

    fn put_char(&mut self, row: usize, col: usize, ch: ScreenChar) {
        if self.grid.set(row, col, ch)
            && let Some(renderer) = &mut self.renderer
        {
            renderer.draw(row, col, ch);
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

fn check(row: usize, col: usize) -> Result<(), OutOfBounds> {
    if row < ROWS && col < COLUMNS {
        Ok(())
    } else {
        Err(OutOfBounds)
    }
}
