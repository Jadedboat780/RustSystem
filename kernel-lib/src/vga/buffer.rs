use super::colors::ColorCode;
use bootloader_api::info::{FrameBuffer, FrameBufferInfo, PixelFormat};
use embedded_graphics::{
    Drawable, Pixel,
    draw_target::DrawTarget,
    geometry::{OriginDimensions, Point, Size},
    mono_font::{
        MonoFont, MonoTextStyleBuilder,
        ascii::{FONT_6X10, FONT_7X14, FONT_8X13, FONT_9X18, FONT_10X20},
    },
    pixelcolor::{Rgb888, RgbColor},
    prelude::Primitive,
    primitives::{PrimitiveStyle, Rectangle},
    text::{Baseline, Text},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScreenChar {
    pub ascii_character: u8,
    pub color_code: ColorCode,
}

pub const BUFFER_HEIGHT: usize = 25;
pub const BUFFER_WIDTH: usize = 80;

const FONTS: [&MonoFont<'static>; 5] =
    [&FONT_10X20, &FONT_9X18, &FONT_8X13, &FONT_7X14, &FONT_6X10];

/// The 16 standard VGA text-mode colors as RGB
const PALETTE: [Rgb888; 16] = [
    Rgb888::new(0x00, 0x00, 0x00), // Black
    Rgb888::new(0x00, 0x00, 0xAA), // Blue
    Rgb888::new(0x00, 0xAA, 0x00), // Green
    Rgb888::new(0x00, 0xAA, 0xAA), // Cyan
    Rgb888::new(0xAA, 0x00, 0x00), // Red
    Rgb888::new(0xAA, 0x00, 0xAA), // Magenta
    Rgb888::new(0xAA, 0x55, 0x00), // Brown
    Rgb888::new(0xAA, 0xAA, 0xAA), // LightGray
    Rgb888::new(0x55, 0x55, 0x55), // DarkGray
    Rgb888::new(0x55, 0x55, 0xFF), // LightBlue
    Rgb888::new(0x55, 0xFF, 0x55), // LightGreen
    Rgb888::new(0x55, 0xFF, 0xFF), // LightCyan
    Rgb888::new(0xFF, 0x55, 0x55), // LightRed
    Rgb888::new(0xFF, 0x55, 0xFF), // Pink
    Rgb888::new(0xFF, 0xFF, 0x55), // Yellow
    Rgb888::new(0xFF, 0xFF, 0xFF), // White
];

pub struct Buffer {
    chars: [[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT],
    display: Option<Display>,
}

impl Buffer {
    pub const fn new(color_code: ColorCode) -> Self {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code,
        };
        Self {
            chars: [[blank; BUFFER_WIDTH]; BUFFER_HEIGHT],
            display: None,
        }
    }

    pub fn attach(&mut self, framebuffer: &'static mut FrameBuffer) {
        let mut display = Display::new(framebuffer);
        display.clear_screen();
        for (row, line) in self.chars.iter().enumerate() {
            for (col, &ch) in line.iter().enumerate() {
                display.draw_char(row, col, ch);
            }
        }
        self.display = Some(display);
    }

    pub fn read(&self, row: usize, col: usize) -> ScreenChar {
        self.chars[row][col]
    }

    pub fn write(&mut self, row: usize, col: usize, ch: ScreenChar) {
        if self.chars[row][col] == ch {
            return;
        }
        self.chars[row][col] = ch;
        if let Some(display) = &mut self.display {
            display.draw_char(row, col, ch);
        }
    }

    pub fn scroll_up(&mut self) {
        self.chars.copy_within(1.., 0);
        if let Some(display) = &mut self.display {
            display.scroll_up();
        }
    }
}

struct Display {
    pixels: &'static mut [u8],
    info: FrameBufferInfo,
    font: &'static MonoFont<'static>,
    scale: usize,
    origin_x: usize,
    origin_y: usize,
}

impl Display {
    fn new(framebuffer: &'static mut FrameBuffer) -> Self {
        let info = framebuffer.info();
        let (font, scale) = choose_font(&info);
        let grid_width = BUFFER_WIDTH * cell_size(font).width as usize * scale;
        let grid_height = BUFFER_HEIGHT * cell_size(font).height as usize * scale;
        Self {
            pixels: framebuffer.buffer_mut(),
            info,
            font,
            scale,
            origin_x: info.width.saturating_sub(grid_width) / 2,
            origin_y: info.height.saturating_sub(grid_height) / 2,
        }
    }

    fn cell(&self) -> Size {
        cell_size(self.font)
    }

    fn clear_screen(&mut self) {
        self.pixels.fill(0);
    }

    fn draw_char(&mut self, row: usize, col: usize, ch: ScreenChar) {
        let fg = PALETTE[ch.color_code.foreground() as usize];
        let bg = PALETTE[ch.color_code.background() as usize];
        let cell = self.cell();
        let top_left = Point::new(
            (col as u32 * cell.width) as i32,
            (row as u32 * cell.height) as i32,
        );

        let byte = [ch.ascii_character];
        let _ = match core::str::from_utf8(&byte) {
            Ok(text) => {
                let style = MonoTextStyleBuilder::new()
                    .font(self.font)
                    .text_color(fg)
                    .background_color(bg)
                    .build();
                Text::with_baseline(text, top_left, style, Baseline::Top)
                    .draw(self)
                    .map(|_| ())
            }
            Err(_) => Rectangle::new(top_left, cell)
                .into_styled(PrimitiveStyle::with_fill(fg))
                .draw(self),
        };
    }

    fn put_pixel(&mut self, x: usize, y: usize, color: Rgb888) {
        if x >= self.info.width || y >= self.info.height {
            return;
        }
        let (r, g, b) = (color.r(), color.g(), color.b());
        let bpp = self.info.bytes_per_pixel;
        let offset = (y * self.info.stride + x) * bpp;
        let bytes = match self.info.pixel_format {
            PixelFormat::Rgb => [r, g, b, 0],
            PixelFormat::U8 => [((r as u16 + g as u16 + b as u16) / 3) as u8, 0, 0, 0],
            _ => [b, g, r, 0], // Bgr, and the most common layout for unknown formats
        };
        let len = bpp.min(bytes.len());
        self.pixels[offset..offset + len].copy_from_slice(&bytes[..len]);
    }

    fn scroll_up(&mut self) {
        let row_bytes = self.info.stride * self.info.bytes_per_pixel;
        let line = self.cell().height as usize * self.scale * row_bytes;
        let start = self.origin_y * row_bytes;
        let end = start + BUFFER_HEIGHT * line;
        self.pixels.copy_within(start + line..end, start);
    }
}

impl OriginDimensions for Display {
    fn size(&self) -> Size {
        let cell = self.cell();
        Size::new(
            cell.width * BUFFER_WIDTH as u32,
            cell.height * BUFFER_HEIGHT as u32,
        )
    }
}

impl DrawTarget for Display {
    type Color = Rgb888;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(point, color) in pixels {
            let (Ok(x), Ok(y)) = (usize::try_from(point.x), usize::try_from(point.y)) else {
                continue;
            };
            let x0 = self.origin_x + x * self.scale;
            let y0 = self.origin_y + y * self.scale;
            for dy in 0..self.scale {
                for dx in 0..self.scale {
                    self.put_pixel(x0 + dx, y0 + dy, color);
                }
            }
        }
        Ok(())
    }
}

fn cell_size(font: &MonoFont) -> Size {
    Size::new(
        font.character_size.width + font.character_spacing,
        font.character_size.height,
    )
}

fn choose_font(info: &FrameBufferInfo) -> (&'static MonoFont<'static>, usize) {
    let mut best = (FONTS[FONTS.len() - 1], 1, 0);
    for font in FONTS {
        let cell = cell_size(font);
        let (w, h) = (cell.width as usize, cell.height as usize);
        let scale = (info.width / (BUFFER_WIDTH * w)).min(info.height / (BUFFER_HEIGHT * h));
        let area = w * h * scale * scale;
        if area > best.2 {
            best = (font, scale, area);
        }
    }
    (best.0, best.1)
}
