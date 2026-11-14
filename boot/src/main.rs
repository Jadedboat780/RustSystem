//! Cargo runner for the kernel (see `.cargo/config.toml`).

use std::{
    env,
    path::{Path, PathBuf},
    process::{Command, ExitCode},
    thread,
    time::{Duration, Instant},
};

const TEST_SUCCESS_EXIT_CODE: i32 = 33;
const TEST_TIMEOUT: Duration = Duration::from_secs(10);
const OVMF_CANDIDATES: &[(&str, &str)] = &[
    (
        "/usr/share/edk2/ovmf/OVMF_CODE.fd",
        "/usr/share/edk2/ovmf/OVMF_VARS.fd",
    ), // Fedora
    (
        "/usr/share/OVMF/OVMF_CODE_4M.fd",
        "/usr/share/OVMF/OVMF_VARS_4M.fd",
    ), // Debian/Ubuntu
    (
        "/usr/share/OVMF/OVMF_CODE.fd",
        "/usr/share/OVMF/OVMF_VARS.fd",
    ),
    (
        "/usr/share/edk2/x64/OVMF_CODE.4m.fd",
        "/usr/share/edk2/x64/OVMF_VARS.4m.fd",
    ), // Arch
];

fn main() -> ExitCode {
    let mut args = env::args_os().skip(1);
    let Some(kernel) = args.next().map(PathBuf::from) else {
        eprintln!("usage: boot <kernel-elf> [qemu args...]");
        return ExitCode::FAILURE;
    };
    let extra_args: Vec<_> = args.collect();

    let is_test = is_test_binary(&kernel);
    let bios = env::var("BOOT").is_ok_and(|v| v.eq_ignore_ascii_case("bios"));

    let mut qemu = Command::new("qemu-system-x86_64");
    if bios {
        let image = kernel.with_extension("bios.img");
        bootloader::BiosBoot::new(&kernel)
            .create_disk_image(&image)
            .expect("failed to create BIOS disk image");
        qemu.arg("-drive")
            .arg(format!("format=raw,file={}", image.display()));
    } else {
        let image = kernel.with_extension("uefi.img");
        bootloader::UefiBoot::new(&kernel)
            .create_disk_image(&image)
            .expect("failed to create UEFI disk image");
        let (code, vars) = ovmf_firmware(&kernel);
        qemu.arg("-drive")
            .arg(format!(
                "if=pflash,format=raw,readonly=on,file={}",
                code.display()
            ))
            .arg("-drive")
            .arg(format!("if=pflash,format=raw,file={}", vars.display()))
            .arg("-drive")
            .arg(format!("format=raw,file={}", image.display()));
    }
    qemu.args(["-serial", "stdio"]);

    if is_test {
        qemu.args(["-device", "isa-debug-exit,iobase=0xf4,iosize=0x04"])
            .args(["-display", "none"]);
    }
    qemu.args(&extra_args);

    let mut child = qemu.spawn().expect("failed to start qemu-system-x86_64");

    if !is_test {
        let status = child.wait().expect("failed to wait for QEMU");
        return if status.success() {
            ExitCode::SUCCESS
        } else {
            ExitCode::FAILURE
        };
    }

    let start = Instant::now();
    loop {
        if let Some(status) = child.try_wait().expect("failed to wait for QEMU") {
            return match status.code() {
                Some(TEST_SUCCESS_EXIT_CODE) => ExitCode::SUCCESS,
                code => {
                    eprintln!("test failed: QEMU exited with {code:?}");
                    ExitCode::FAILURE
                }
            };
        }
        if start.elapsed() > TEST_TIMEOUT {
            let _ = child.kill();
            let _ = child.wait();
            eprintln!("test timed out after {TEST_TIMEOUT:?}");
            return ExitCode::FAILURE;
        }
        thread::sleep(Duration::from_millis(50));
    }
}

fn is_test_binary(kernel: &Path) -> bool {
    let in_deps = kernel
        .parent()
        .and_then(Path::file_name)
        .is_some_and(|dir| dir == "deps");
    let has_hash = kernel
        .file_name()
        .and_then(|name| name.to_str())
        .and_then(|name| name.rsplit_once('-'))
        .is_some_and(|(_, hash)| hash.len() == 16 && hash.bytes().all(|b| b.is_ascii_hexdigit()));
    in_deps || has_hash
}

fn ovmf_firmware(kernel: &Path) -> (PathBuf, PathBuf) {
    let (code, vars) = match (env::var_os("OVMF_CODE"), env::var_os("OVMF_VARS")) {
        (Some(code), Some(vars)) => (PathBuf::from(code), PathBuf::from(vars)),
        _ => OVMF_CANDIDATES
            .iter()
            .map(|&(code, vars)| (PathBuf::from(code), PathBuf::from(vars)))
            .find(|(code, vars)| code.exists() && vars.exists())
            .unwrap_or_else(prebuilt_ovmf),
    };

    let vars_copy = kernel.with_extension("ovmf_vars.fd");
    std::fs::copy(&vars, &vars_copy).expect("failed to copy OVMF_VARS");
    (code, vars_copy)
}

fn prebuilt_ovmf() -> (PathBuf, PathBuf) {
    use ovmf_prebuilt::{Arch, FileType, Prebuilt, Source};

    let cache_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("target/ovmf");
    let prebuilt = Prebuilt::fetch(Source::LATEST, &cache_dir).expect(
        "no system OVMF found and downloading it failed; \
         install edk2-ovmf or set OVMF_CODE and OVMF_VARS",
    );
    (
        prebuilt.get_file(Arch::X64, FileType::Code),
        prebuilt.get_file(Arch::X64, FileType::Vars),
    )
}
