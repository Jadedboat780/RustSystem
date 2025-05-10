use super::colors::ColorCode;
use volatile::Volatile;

//структура для представления символа
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct ScreenChar {
    pub ascii_character: u8,
    pub color_code: ColorCode,
}

//эти значения желательно никогда не менять
pub const BUFFER_HEIGHT: usize = 25; //высота буфера
pub const BUFFER_WIDTH: usize = 80; //ширина буфера

//структура для представления текстового буфера(поддержиавет лишь ASCII символы)
pub struct Buffer {
    pub chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}
