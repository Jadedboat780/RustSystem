//! System calls, entered with `int 0x80`.
//!
//! Calling convention (the Linux x86_64 one, so it keeps working once the
//! `syscall` instruction replaces `int 0x80`; that instruction clobbers RCX and R11):
//! - RAX: syscall number ([`Syscall`])
//! - RDI, RSI, RDX, R10, R8, R9: arguments 1-6
//! - RAX on return: a non-negative result, or a negative error code (`-ENOSYS`, `-EINVAL`)
//!
//! All other registers are preserved

use crate::WRITER;
use core::{arch::global_asm, sync::atomic::Ordering};
use kernel_lib::{
    heap::fixed_block::HEAP_SIZE,
    time::TICKS,
    vga::{
        buffer::{BUFFER_HEIGHT, BUFFER_WIDTH},
        colors::ColorCode,
    },
};

/// Invalid argument
pub const EINVAL: i64 = 22;
/// Unknown syscall number
pub const ENOSYS: i64 = 38;
/// Longest string accepted by [`Syscall::WriteStr`]: one screen row
pub const MAX_WRITE_LEN: usize = BUFFER_WIDTH;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u64)]
pub enum Syscall {
    /// `(col, row, byte, color)`: put one character on the screen
    WriteChar = 0,
    /// `(col, row, ptr, len, color)`: put a UTF-8 string of at most
    /// [`MAX_WRITE_LEN`] bytes on the screen
    WriteStr = 1,
    /// `()`: timer ticks since boot
    GetTicks = 2,
    /// `()`: size of the kernel heap in bytes
    HeapSize = 0x10,
}

impl TryFrom<u64> for Syscall {
    type Error = ();

    fn try_from(num: u64) -> Result<Self, Self::Error> {
        Ok(match num {
            0 => Self::WriteChar,
            1 => Self::WriteStr,
            2 => Self::GetTicks,
            0x10 => Self::HeapSize,
            _ => return Err(()),
        })
    }
}

#[unsafe(no_mangle)]
extern "C" fn syscall_handler(num: u64, arg1: u64, arg2: u64, arg3: u64, arg4: u64, arg5: u64, _arg6: u64) -> i64 {
    let Ok(syscall) = Syscall::try_from(num) else {
        return -ENOSYS;
    };
    let result = match syscall {
        Syscall::WriteChar => write_char(arg1, arg2, arg3, arg4),
        Syscall::WriteStr => write_str(arg1, arg2, arg3, arg4, arg5),
        Syscall::GetTicks => Ok(TICKS.load(Ordering::Relaxed) as i64),
        Syscall::HeapSize => Ok(HEAP_SIZE as i64),
    };
    result.unwrap_or_else(|errno| -errno)
}

/// Checks screen coordinates and converts them to `(row, col)`
fn position(col: u64, row: u64) -> Result<(usize, usize), i64> {
    let (row, col) = (row as usize, col as usize);
    if row < BUFFER_HEIGHT && col < BUFFER_WIDTH {
        Ok((row, col))
    } else {
        Err(EINVAL)
    }
}

fn color(raw: u64) -> Result<ColorCode, i64> {
    ColorCode::try_from(raw).map_err(|_| EINVAL)
}

fn write_char(col: u64, row: u64, byte: u64, color_code: u64) -> Result<i64, i64> {
    let (row, col) = position(col, row)?;
    let byte = u8::try_from(byte).map_err(|_| EINVAL)?;
    let color_code = color(color_code)?;
    WRITER.lock().write_byte_at(row, col, byte, color_code);
    Ok(0)
}

fn write_str(col: u64, row: u64, ptr: u64, len: u64, color_code: u64) -> Result<i64, i64> {
    let (row, col) = position(col, row)?;
    let color_code = color(color_code)?;
    let len = usize::try_from(len).map_err(|_| EINVAL)?;
    if len > MAX_WRITE_LEN || (ptr == 0 && len > 0) {
        return Err(EINVAL);
    }
    let bytes = unsafe { core::slice::from_raw_parts(ptr as *const u8, len) };
    let text = core::str::from_utf8(bytes).map_err(|_| EINVAL)?;
    WRITER.lock().write_string_at(row, col, text, color_code);
    Ok(len as i64)
}

global_asm!(
    r#"
.globl syscall_entry
.text
syscall_entry:
    // Save the caller's registers. After all pushes the layout is:
    // [rsp+64] rax, [rsp+56] rdi, [rsp+48] rsi, [rsp+40] rdx, [rsp+32] rcx,
    // [rsp+24] r8,  [rsp+16] r9,  [rsp+8]  r10, [rsp+0]  r11
    push %rax
    push %rdi
    push %rsi
    push %rdx
    push %rcx
    push %r8
    push %r9
    push %r10
    push %r11

    // Pass arg6 (saved r9) as the 7th System V argument, on the stack.
    // The extra 8 bytes keep RSP 16-byte aligned at the call.
    sub $8, %rsp
    pushq 24(%rsp)       // saved r9, now 16 bytes further down

    // Move the arguments into the System V order expected by `syscall_handler`
    // (offsets are 16 bytes larger than in the layout above)
    mov 80(%rsp), %rdi   // num  <- rax
    mov 72(%rsp), %rsi   // arg1 <- rdi
    mov 64(%rsp), %rdx   // arg2 <- rsi
    mov 56(%rsp), %rcx   // arg3 <- rdx
    mov 24(%rsp), %r8    // arg4 <- r10
    mov 40(%rsp), %r9    // arg5 <- r8

    // The System V ABI requires the direction flag to be clear on calls,
    // and an interrupt gate does not clear it.
    cld

    // PC-relative call: the kernel is position-independent
    call syscall_handler
    add $16, %rsp

    // Store the return value in the saved RAX slot so the caller gets it
    mov %rax, 64(%rsp)

    // Restore registers
    pop %r11
    pop %r10
    pop %r9
    pop %r8
    pop %rcx
    pop %rdx
    pop %rsi
    pop %rdi
    pop %rax

    iretq
"#,
    options(att_syntax)
);

unsafe extern "C" {
    pub fn syscall_entry();
}

/// Issues a syscall with the convention described at the top of this module
#[cfg(test)]
fn syscall(num: Syscall, args: [u64; 5]) -> i64 {
    let result: i64;
    unsafe {
        core::arch::asm!(
            "int 0x80",
            inlateout("rax") num as u64 => result,
            in("rdi") args[0],
            in("rsi") args[1],
            in("rdx") args[2],
            in("r10") args[3],
            in("r8") args[4],
        );
    }
    result
}

#[cfg(test)]
const TEST_COLOR: u64 = 0x0D; // pink on black

#[test_case]
fn test_heap_size() {
    assert_eq!(syscall(Syscall::HeapSize, [0; 5]), HEAP_SIZE as i64);
}

#[test_case]
fn test_get_ticks_is_monotonic() {
    let first = syscall(Syscall::GetTicks, [0; 5]);
    let second = syscall(Syscall::GetTicks, [0; 5]);
    assert!(first >= 0 && second >= first);
}

#[test_case]
fn test_unknown_syscall() {
    let result: i64;
    unsafe {
        core::arch::asm!("int 0x80", inlateout("rax") 0xDEADu64 => result);
    }
    assert_eq!(result, -ENOSYS);
}

#[test_case]
fn test_syscall_preserves_registers() {
    let (rdi, rsi, rdx, rcx, r8, r9, r10, r11): (u64, u64, u64, u64, u64, u64, u64, u64);
    unsafe {
        core::arch::asm!(
            "int 0x80",
            inlateout("rax") Syscall::GetTicks as u64 => _,
            inlateout("rdi") 11u64 => rdi,
            inlateout("rsi") 22u64 => rsi,
            inlateout("rdx") 33u64 => rdx,
            inlateout("rcx") 44u64 => rcx,
            inlateout("r8") 55u64 => r8,
            inlateout("r9") 66u64 => r9,
            inlateout("r10") 77u64 => r10,
            inlateout("r11") 88u64 => r11,
        );
    }
    assert_eq!((rdi, rsi, rdx, rcx, r8, r9, r10, r11), (11, 22, 33, 44, 55, 66, 77, 88));
}

#[test_case]
fn test_write_char() {
    assert_eq!(syscall(Syscall::WriteChar, [5, 3, b'X' as u64, TEST_COLOR, 0]), 0);
    let ch = WRITER.lock().char_at(3, 5).unwrap();
    assert_eq!(ch.ascii_character, b'X');
    assert_eq!(ch.color_code, ColorCode::try_from(TEST_COLOR).unwrap());
}

#[test_case]
fn test_write_str() {
    let text = "hello";
    let args = [10, 4, text.as_ptr() as u64, text.len() as u64, TEST_COLOR];
    assert_eq!(syscall(Syscall::WriteStr, args), text.len() as i64);
    let writer = WRITER.lock();
    for (i, byte) in text.bytes().enumerate() {
        assert_eq!(writer.char_at(4, 10 + i).unwrap().ascii_character, byte);
    }
}

#[test_case]
fn test_write_rejects_invalid_arguments() {
    let text = [b'a'; MAX_WRITE_LEN + 1];
    let ptr = text.as_ptr() as u64;
    #[rustfmt::skip]
    let invalid = [
        (Syscall::WriteChar, [BUFFER_WIDTH as u64, 0, b'a' as u64, TEST_COLOR, 0]), // column
        (Syscall::WriteChar, [0, BUFFER_HEIGHT as u64, b'a' as u64, TEST_COLOR, 0]), // row
        (Syscall::WriteChar, [0, 0, 0x100, TEST_COLOR, 0]),                         // byte
        (Syscall::WriteChar, [0, 0, b'a' as u64, 0x100, 0]),                        // color
        (Syscall::WriteStr, [0, 0, ptr, text.len() as u64, TEST_COLOR]),            // too long
        (Syscall::WriteStr, [0, 0, 0, 1, TEST_COLOR]),                              // null ptr
        (Syscall::WriteStr, [0, 0, [0xFFu8].as_ptr() as u64, 1, TEST_COLOR]),       // not UTF-8
    ];
    for (num, args) in invalid {
        assert_eq!(syscall(num, args), -EINVAL, "{num:?} {args:?}");
    }
}
