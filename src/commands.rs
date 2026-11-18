use crate::{WRITER, hlt_loop, keyboard, print, println};
use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use core::arch::asm;
use kernel_lib::{console::COLUMNS, time};
use x86_64::instructions::port::Port;

const BUBBLE_WIDTH: usize = 40;

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Help,
    Version,
    Reboot,
    Shutdown,
    Clear,
    Ferris(String),
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
            Ferris(text) => ferris(text),
            Error(command) => error_command(command),
        }
        print!("{}$ ", time::now());
    }
}

impl From<&str> for Command {
    fn from(val: &str) -> Self {
        use Command::*;

        let val = val.trim();
        let (name, args) = val.split_once(' ').unwrap_or((val, ""));
        match name {
            "help" => Help,
            "version" => Version,
            "reboot" => Reboot,
            "shutdown" => Shutdown,
            "clear" => Clear,
            "ferris" => Ferris(args.trim().to_string()),
            _ => Error(val.to_string()),
        }
    }
}

fn help() {
    println!(
        ">>> Available commands:
    version   - Display OS version information
    reboot    - Reboot the system
    shutdown  - Power off the system
    clear     - Clear the screen
    ferris    - Let Ferris the crab say something: ferris [text]
    "
    );
}

fn version() {
    println!(">>> Actual version: {}\n", env!("CARGO_PKG_VERSION"))
}

pub fn reboot_action() {
    {
        WRITER.lock().write_string("\nRebooting...");
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
        WRITER.lock().write_string("\nShutting down...");
    }

    x86_64::instructions::interrupts::disable();
    for (port, value) in [(0x604, 0x2000), (0xB004, 0x2000), (0x4004, 0x3400)] {
        unsafe { Port::<u16>::new(port).write(value) };
    }
    hlt_loop()
}

pub fn clear() {
    WRITER.lock().clear_screen()
}

fn error_command(command: &str) {
    println!(">>> Command not found: {}\n", command);
}

pub fn show_splash() {
    const ASCII_LOGO: &str = r"
         ____            _     ____            _
        |  _ \ _   _ ___| |_  / ___| _   _ ___| |_ ___ _ __ ___
        | |_) | | | / __| __| \___ \| | | / __| __/ _ \ '_ ` _ \
        |  _ <| |_| \__ \ |_   ___) | |_| \__ \ ||  __/ | | | | |
        |_| \_\\__,_|___/\__| |____/ \__, |___/\__\___|_| |_| |_|
                                     |___/
";
    const PROMPT: &str = "Press Enter to continue";

    clear();
    println!("{}", ASCII_LOGO);
    println!("\n{:>width$}", PROMPT, width = (COLUMNS + PROMPT.len()) / 2);
    keyboard::wait_for_enter();
    clear();
}

fn ferris(text: &str) {
    const FERRIS: &str = r"        \
         \
            _~^~^~_
        \) /  o o  \ (/
          '_   -   _'
          / '-----' \";

    let text = if text.is_empty() { "Hello, fellow Rustaceans!" } else { text };
    let lines = wrap(text, BUBBLE_WIDTH);
    let width = lines.iter().map(|line| line.chars().count()).max().unwrap_or(0);

    println!(" {}", "_".repeat(width + 2));
    for (i, line) in lines.iter().enumerate() {
        let (left, right) = match (lines.len(), i) {
            (1, _) => ('<', '>'),
            (_, 0) => ('/', '\\'),
            (n, i) if i == n - 1 => ('\\', '/'),
            _ => ('|', '|'),
        };
        println!("{} {:<width$} {}", left, line, right);
    }
    println!(" {}", "-".repeat(width + 2));
    println!("{}\n", FERRIS);
}

fn wrap(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = String::new();
    for mut word in text.split_whitespace() {
        while word.chars().count() > width {
            let split = word.char_indices().nth(width).map_or(word.len(), |(i, _)| i);
            if !line.is_empty() {
                lines.push(core::mem::take(&mut line));
            }
            lines.push(word[..split].to_string());
            word = &word[split..];
        }
        if !line.is_empty() && line.chars().count() + 1 + word.chars().count() > width {
            lines.push(core::mem::take(&mut line));
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(word);
    }
    if !line.is_empty() || lines.is_empty() {
        lines.push(line);
    }
    lines
}

#[test_case]
fn test_parse_commands() {
    assert_eq!(Command::from("help"), Command::Help);
    assert_eq!(Command::from("  clear "), Command::Clear);
    assert_eq!(Command::from("ferris"), Command::Ferris(String::new()));
    assert_eq!(Command::from("ferris hi  there "), Command::Ferris("hi  there".to_string()));
    assert_eq!(Command::from("nope x"), Command::Error("nope x".to_string()));
}

#[test_case]
fn test_wrap() {
    assert_eq!(wrap("", 10), [""]);
    assert_eq!(wrap("hello world", 20), ["hello world"]);
    assert_eq!(wrap("hello brave new world", 11), ["hello brave", "new world"]);
    assert_eq!(wrap("abcdefghijkl", 5), ["abcde", "fghij", "kl"]);
    assert_eq!(wrap("hi abcdefg", 4), ["hi", "abcd", "efg"]);
}
