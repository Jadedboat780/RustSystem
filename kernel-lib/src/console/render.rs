use super::{
    color::Color,
    grid::{COLUMNS, ROWS, ScreenChar},
};
use bootloader_api::info::{FrameBuffer, FrameBufferInfo, PixelFormat};
use embedded_graphics::{
    geometry::{OriginDimensions, Point},
    image::GetPixel,
    mono_font::{
        MonoFont,
        ascii::{FONT_6X10, FONT_7X14, FONT_8X13, FONT_9X18, FONT_10X20},
    },
    pixelcolor::BinaryColor,
};

const FONTS: [&MonoFont<'static>; 5] = [&FONT_10X20, &FONT_9X18, &FONT_8X13, &FONT_7X14, &FONT_6X10];
const FIRST_GLYPH: u8 = 0x20;
const LAST_GLYPH: u8 = 0x7E;
const GLYPH_COUNT: usize = (LAST_GLYPH - FIRST_GLYPH + 1) as usize;
const MAX_GLYPH_HEIGHT: usize = 32;
const LINE_BUFFER_SIZE: usize = 512;

pub(crate) struct Renderer {
    pixels: &'static mut [u8],
    info: FrameBufferInfo,
    glyphs: [[u16; MAX_GLYPH_HEIGHT]; GLYPH_COUNT],
    cell_width: usize,
    cell_height: usize,
    scale: usize,
    origin_x: usize,
    origin_y: usize,
}

impl Renderer {
    pub fn new(framebuffer: &'static mut FrameBuffer) -> Self {
        let info = framebuffer.info();
        let (font, scale) = choose_font(&info);
        let (cell_width, cell_height) = cell_size(font);
        Self {
            pixels: framebuffer.buffer_mut(),
            info,
            glyphs: rasterize(font),
            cell_width,
            cell_height,
            scale,
            origin_x: info.width.saturating_sub(COLUMNS * cell_width * scale) / 2,
            origin_y: info.height.saturating_sub(ROWS * cell_height * scale) / 2,
        }
    }

    pub fn clear(&mut self) {
        self.pixels.fill(0);
    }

    pub fn draw(&mut self, row: usize, col: usize, ch: ScreenChar) {
        let foreground = self.encode(ch.color_code.foreground());
        let background = self.encode(ch.color_code.background());
        let glyph = match ch.ascii_character {
            byte @ FIRST_GLYPH..=LAST_GLYPH => Some(&self.glyphs[(byte - FIRST_GLYPH) as usize]),
            _ => None,
        };

        let bytes_per_pixel = self.info.bytes_per_pixel;
        let copied = bytes_per_pixel.min(4);
        let row_bytes = self.info.stride * bytes_per_pixel;
        let x0 = self.origin_x + col * self.cell_width * self.scale;
        let y0 = self.origin_y + row * self.cell_height * self.scale;
        let visible_width = self.info.width.saturating_sub(x0).min(self.cell_width * self.scale);
        let line_len = visible_width * bytes_per_pixel;

        let mut line = [0u8; LINE_BUFFER_SIZE];
        for glyph_y in 0..self.cell_height {
            let bits = glyph.map_or(u16::MAX, |glyph| glyph[glyph_y]);
            for x in 0..visible_width {
                let color = if bits >> (x / self.scale) & 1 == 1 {
                    &foreground
                } else {
                    &background
                };
                let offset = x * bytes_per_pixel;
                line[offset..offset + copied].copy_from_slice(&color[..copied]);
            }

            for sub_y in 0..self.scale {
                let y = y0 + glyph_y * self.scale + sub_y;
                if y >= self.info.height {
                    return;
                }
                let start = y * row_bytes + x0 * bytes_per_pixel;
                self.pixels[start..start + line_len].copy_from_slice(&line[..line_len]);
            }
        }
    }

    fn encode(&self, color: Color) -> [u8; 4] {
        let (r, g, b) = color.rgb();
        match self.info.pixel_format {
            PixelFormat::Rgb => [r, g, b, 0],
            PixelFormat::U8 => [((r as u16 + g as u16 + b as u16) / 3) as u8, 0, 0, 0],
            _ => [b, g, r, 0],
        }
    }
}

fn rasterize(font: &MonoFont) -> [[u16; MAX_GLYPH_HEIGHT]; GLYPH_COUNT] {
    let width = font.character_size.width;
    let height = font.character_size.height;
    let glyphs_per_row = font.image.size().width / width;
    let mut glyphs = [[0u16; MAX_GLYPH_HEIGHT]; GLYPH_COUNT];
    for (i, glyph) in glyphs.iter_mut().enumerate() {
        let index = font.glyph_mapping.index((FIRST_GLYPH + i as u8) as char) as u32;
        let origin = Point::new(
            ((index % glyphs_per_row) * width) as i32,
            ((index / glyphs_per_row) * height) as i32,
        );
        for (y, bits) in glyph.iter_mut().enumerate().take(height as usize) {
            for x in 0..width {
                if font.image.pixel(origin + Point::new(x as i32, y as i32)) == Some(BinaryColor::On) {
                    *bits |= 1 << x;
                }
            }
        }
    }
    glyphs
}

fn cell_size(font: &MonoFont) -> (usize, usize) {
    (
        (font.character_size.width + font.character_spacing) as usize,
        font.character_size.height as usize,
    )
}

fn choose_font(info: &FrameBufferInfo) -> (&'static MonoFont<'static>, usize) {
    let mut best = (FONTS[FONTS.len() - 1], 1, 0);
    for font in FONTS {
        let (width, height) = cell_size(font);
        let max_scale = LINE_BUFFER_SIZE / (width * info.bytes_per_pixel.max(1));
        let scale = (info.width / (COLUMNS * width))
            .min(info.height / (ROWS * height))
            .min(max_scale);
        let area = width * height * scale * scale;
        if area > best.2 {
            best = (font, scale, area);
        }
    }
    (best.0, best.1)
}
