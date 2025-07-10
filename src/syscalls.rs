use crate::WRITER;
use allocators::fixed_block::HEAP_SIZE;
use core::{arch::global_asm, sync::atomic::Ordering};
use datetime::TICKS;
use vga::colors::{Color, ColorCode};

#[unsafe(no_mangle)]
pub extern "C" fn syscall_handler(
    num: u64,
    arg1: u64,
    arg2: u64,
    arg3: u64,
    arg4: u64,
    arg5: u64,
    arg6: u64,
) -> u64 {
    match num {
        0 => {
            WRITER.lock().write_byte_at(
                arg2 as usize,
                arg1 as usize,
                arg3 as u8,
                ColorCode::from(arg4),
            );
            0
        }
        1 => {
            unsafe {
                let ptr = arg3 as *const u8;

                let mut len = 0;
                while *ptr.add(len) != 0 {
                    len += 1;
                }

                let slice = core::slice::from_raw_parts(ptr, len);
                let mut writer = WRITER.lock();

                if let Ok(s) = core::str::from_utf8(slice) {
                    writer.write_string_at(arg2 as usize, arg1 as usize, s, ColorCode::from(arg4));
                } else {
                    writer.write_string_at(
                        arg2 as usize,
                        arg1 as usize,
                        "UTF8 Err",
                        ColorCode::new(Color::Red, Color::Yellow),
                    );
                }
            }
            0
        }
        2 => TICKS.load(Ordering::Relaxed) as u64,
        0x10 => HEAP_SIZE as u64,
        _ => 0,
    }
}

global_asm!(
    r#"
    .att_syntax
.globl syscall_entry
.text
syscall_entry:
    // Save the registers
    push %rax      // [rsp+0]
    push %rdi      // [rsp+8]
    push %rsi      // [rsp+16]
    push %rdx      // [rsp+24]
    push %rcx      // [rsp+32]
    push %r8       // [rsp+40]
    push %r9       // [rsp+48]
    push %r10
    push %r11

    // Unpacking the arguments on the stack
    mov 64(%rsp), %rdi   // syscall number (из RAX)
    mov 56(%rsp), %rsi   // x (из RDI)
    mov 48(%rsp), %rdx   // y (из RSI)
    mov 40(%rsp), %rcx   // ch (из RDX)
    mov 32(%rsp), %r8    // color (из RCX)
    mov 24(%rsp), %r9    // 6-й аргумент (опц.)

    // Calling the handler
    mov $syscall_handler, %rax
    call *%rax

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

"#
);

unsafe extern "C" {
    pub fn syscall_entry();
}
