use crate::{WRITER, print, println};
use alloc::string::{String, ToString};
use core::arch::asm;
use datetime::DateTime;

pub enum Command {
    Help,
    Version,
    Reboot,
    Shutdown,
    Clear,
    Error(String),
}

impl Command {
    pub fn execute(&self) {
        use Command::*;
        print!("\n");
        match self {
            Help => help(),
            Version => version(),
            Reboot => reboot_action(),
            Shutdown => shutdown_action(),
            Clear => clear(),
            Error(command) => error_command(command),
        }
        print!("{}$ ", DateTime::now());
    }
}

impl From<&str> for Command {
    fn from(val: &str) -> Self {
        use Command::*;

        match val {
            "help" => Help,
            "version" => Version,
            "reboot" => Reboot,
            "shutdown" => Shutdown,
            "clear" => Clear,
            _ => Error(val.to_string()),
        }
    }
}

// pub fn command(command: Command) {
//     use Command::*;
//
//     print!("\n");
//     match command {
//         Help => help(),
//         Version => version(),
//         Reboot => reboot_action(),
//         Shutdown => shutdown_action(),
//         Clear => clear(),
//         Error(ref command) => error_command(command),
//     }
//     print!("{}$ ", DateTime::now());
// }

fn help() {
    println!(
        ">>> Available commands:
    version   - Display OS version information
    reboot    - Reboot the system
    shutdown  - Power off the system
    clear     - Clear the screen
    "
    );
}

fn version() {
    println!(">>> Actual version: {}\n", env!("CARGO_PKG_VERSION"))
}

pub fn reboot_action() {
    {
        let mut writer = WRITER.lock();
        writer.set_column_position(0);
        writer.write_string("\n");
        writer.write_string("Rebooting...");
    }

    unsafe {
        asm!(
        "cli",            // Disconnect the interruption
        "out 0x64, al",   // Send the command to the keyboard controller
        "2: hlt",         // Label 2: stop the processor
        "jmp 2b",         // Transition to label 2 to create an endless cycle
        in("al") 0xFEu8   // 0xfe value for the reboot command
        );
    }
}

fn shutdown_action() -> ! {
    {
        let mut writer = WRITER.lock();
        writer.set_column_position(0);
        writer.write_string("\n");
        writer.write_string("Shutting down...");
    }

    unsafe {
        asm!(
            "cli",            // Disconnect the interruption
            "mov ax, 0x5301", // Connect to APM API
            "xor bx, bx",
            "int 0x15",
            "mov ax, 0x530E", // Install the version of APM on 1.2
            "xor bx, bx",
            "mov cx, 0x0102",
            "int 0x15",
            "mov ax, 0x5307", // Turn off the system
            "mov bx, 0x0001",
            "mov cx, 0x0003",
            "int 0x15",
            "hlt", // Stop the processor
            options(noreturn, nostack)
        )
    }
}

pub fn clear() {
    WRITER.lock().clear_screen()
}

fn error_command(command: &str) {
    println!(">>> Command not found: {}\n", command);
}
