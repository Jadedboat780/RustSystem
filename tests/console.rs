#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rust_system::test_runner)]
#![reexport_test_harness_main = "test_main"]

use bootloader_api::{BootInfo, entry_point};
use kernel_lib::console::{COLUMNS, Color, ColorCode, OutOfBounds, ROWS, Writer};

entry_point!(main);

fn main(_boot_info: &'static mut BootInfo) -> ! {
    test_main();
    rust_system::hlt_loop();
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    rust_system::test_panic_handler(info)
}

fn byte_at(writer: &Writer, row: usize, col: usize) -> u8 {
    writer.char_at(row, col).expect("cell out of bounds").ascii_character
}

#[test_case]
fn test_color_code_parts() {
    let code = ColorCode::new(Color::Yellow, Color::Blue);
    assert_eq!(code.foreground(), Color::Yellow);
    assert_eq!(code.background(), Color::Blue);
    assert_eq!(Color::from_u4(13), Color::Pink);
    assert_eq!(Color::from_u4(0xF3), Color::Cyan);
}

#[test_case]
fn test_color_code_with_replaces_bits() {
    let code = ColorCode::new(Color::White, Color::Blue).with_background(Color::Green);
    assert_eq!(code.background(), Color::Green);
    assert_eq!(code.foreground(), Color::White);

    let code = code.with_foreground(Color::Black);
    assert_eq!(code, ColorCode::new(Color::Black, Color::Green));
}

#[test_case]
fn test_color_code_try_from() {
    assert!(ColorCode::try_from(0x1Fu64).is_ok());
    assert!(ColorCode::try_from(0x100u64).is_err());
}

#[test_case]
fn test_writer_colors() {
    let mut writer = Writer::new(ColorCode::DEFAULT);
    writer.set_foreground(Color::Green);
    writer.set_background(Color::Blue);
    assert_eq!(writer.color(), ColorCode::new(Color::Green, Color::Blue));

    writer.write_byte(b'x');
    let (row, _) = writer.cursor();
    assert_eq!(writer.char_at(row, 0).map(|c| c.color_code), Some(writer.color()));
}

#[test_case]
fn test_writer_starts_at_bottom_and_scrolls() {
    let mut writer = Writer::new(ColorCode::DEFAULT);
    assert_eq!(writer.cursor(), (ROWS - 1, 0));

    writer.write_string("first\nsecond");
    assert_eq!(byte_at(&writer, ROWS - 2, 0), b'f');
    assert_eq!(byte_at(&writer, ROWS - 1, 0), b's');
    assert_eq!(writer.cursor(), (ROWS - 1, 6));
}

#[test_case]
fn test_writer_wraps_long_lines() {
    let mut writer = Writer::new(ColorCode::DEFAULT);
    for _ in 0..COLUMNS {
        writer.write_byte(b'a');
    }
    writer.write_byte(b'b');
    assert_eq!(byte_at(&writer, ROWS - 2, COLUMNS - 1), b'a');
    assert_eq!(byte_at(&writer, ROWS - 1, 0), b'b');
}

#[test_case]
fn test_writer_tab_aligns() {
    let mut writer = Writer::new(ColorCode::DEFAULT);
    writer.write_string("ab\tc");
    assert_eq!(writer.cursor().1, 5);
    assert_eq!(byte_at(&writer, ROWS - 1, 4), b'c');

    writer.write_string("\t");
    assert_eq!(writer.cursor().1, 8);
}

#[test_case]
fn test_writer_tab_at_line_end_wraps() {
    let mut writer = Writer::new(ColorCode::DEFAULT);
    writer.set_cursor(ROWS - 1, COLUMNS - 2).unwrap();
    writer.write_byte(b'\t');
    assert_eq!(writer.cursor(), (ROWS - 1, 0));
}

#[test_case]
fn test_writer_delete_char() {
    let mut writer = Writer::new(ColorCode::DEFAULT);
    writer.delete_char();
    assert_eq!(writer.cursor().1, 0);

    writer.write_string("ab");
    writer.delete_char();
    assert_eq!(writer.cursor().1, 1);
    assert_eq!(byte_at(&writer, ROWS - 1, 1), b' ');
    assert_eq!(byte_at(&writer, ROWS - 1, 0), b'a');
}

#[test_case]
fn test_writer_replaces_non_printable() {
    let mut writer = Writer::new(ColorCode::DEFAULT);
    writer.write_string("é");
    assert_eq!(byte_at(&writer, ROWS - 1, 0), 0xFE);
    assert_eq!(byte_at(&writer, ROWS - 1, 1), 0xFE);
}

#[test_case]
fn test_writer_write_at() {
    let mut writer = Writer::new(ColorCode::DEFAULT);
    let color = ColorCode::new(Color::Red, Color::Black);

    assert_eq!(writer.write_byte_at(2, 3, b'x', color), Ok(()));
    assert_eq!(
        writer.char_at(2, 3).map(|c| (c.ascii_character, c.color_code)),
        Some((b'x', color))
    );
    assert_eq!(writer.write_byte_at(ROWS, 0, b'x', color), Err(OutOfBounds));
    assert_eq!(writer.write_byte_at(0, COLUMNS, b'x', color), Err(OutOfBounds));

    assert_eq!(writer.write_string_at(1, COLUMNS - 3, "hello", color), Ok(3));
    assert_eq!(byte_at(&writer, 1, COLUMNS - 1), b'l');
    assert_eq!(writer.write_string_at(1, COLUMNS, "x", color), Err(OutOfBounds));
    assert_eq!(writer.char_at(ROWS, 0), None);
}

#[test_case]
fn test_writer_cursor_and_clear() {
    let mut writer = Writer::new(ColorCode::DEFAULT);
    assert_eq!(writer.set_cursor(ROWS, 0), Err(OutOfBounds));

    writer.set_cursor(3, 7).unwrap();
    writer.write_string("hi\nthere");
    assert_eq!(byte_at(&writer, 3, 7), b'h');
    assert_eq!(byte_at(&writer, 4, 0), b't');

    writer.clear_line();
    assert_eq!(writer.cursor(), (4, 0));
    assert_eq!(byte_at(&writer, 4, 0), b' ');
    assert_eq!(byte_at(&writer, 3, 7), b'h');

    writer.clear_screen();
    assert_eq!(writer.cursor(), (0, 0));
    assert_eq!(byte_at(&writer, 3, 7), b' ');
}
