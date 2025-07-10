# Rust System - minimal 64-bit kernel for x86 architecture on Rust
![Rust](https://img.shields.io/badge/rust-1.89.0_nightly-orange.svg)

<video width="480" controls autoplay muted loop src="example.mp4"></video>

**Rust System** is an educational project aimed at exploring systems programming by building a simple operating system kernel in Rust for the x86_64 architecture. The project does not have a final production goal — it’s primarily a learning exercise.
Current version: `0.0.2`

## Launch
**Requirements:**
* [QEMU](https://www.qemu.org/) installed
* [Rust](https://www.rust-lang.org/tools/install) toolchain

**Setup:**
```bash
rustup component add llvm-tools-preview
cargo install bootimage
```

**Build and run:**
```bash
cargo bootimage
cargo run --release
```

## Features
Implemented so far:
* VGA‑based primitive terminal & cli commands 
* Serial port output for debugging
* Interrupt handling (keyboard and PIT timer)
* Virtual memory management using page tables & frame allocator
* Dynamic heap allocator
* CPU exception handling with TSS/double-fault stack
* Datetime system
* System calls
* Kernel-level testing framework
* Integration with `bootloader` and `bootimage`

## Educational materials:
* [Writing an OS in Rust](https://os.phil-opp.com)
* [Rust Atomics and Locks](https://marabos.nl/atomics/)
* Creating your kernel on rust [1](https://habr.com/ru/articles/920554/), [2](https://habr.com/ru/articles/921500/)

## Notes
RustSystem is a work-in-progress and intended primarily for learning. The codebase, structure, and features are subject to change as the project evolves.
Contributions, suggestions, and issues are welcome!

## License

This project is licensed under the [MIT License](LICENSE).

You are free to use, modify, and distribute this software for personal, educational, or commercial purposes, provided the license terms are met.

