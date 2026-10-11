//! Select the bundled LLVM GNU/PE driver and the x64 MinGW DLL entry point.
use std::{
    env,
    ffi::OsString,
    process::{Command, ExitCode},
};
fn main() -> ExitCode {
    let args: Vec<OsString> = env::args_os().skip(1).collect();
    let Some(linker) = env::var_os("OPSEC_RUST_LLD") else {
        eprintln!("OPSEC_RUST_LLD is unset");
        return ExitCode::FAILURE;
    };
    let mut command = Command::new(linker);
    if args.first().is_none_or(|arg| arg != "-flavor") {
        command.args(["-flavor", "gnu"]);
    }
    command.args(&args);
    // rust-lld's GNU driver otherwise defaults to the 32-bit underscored symbol.
    let shared = args.iter().any(|arg| {
        if arg == "-shared" {
            return true;
        }
        let text = arg.to_string_lossy();
        let Some(path) = text.strip_prefix('@') else {
            return false;
        };
        let Ok(bytes) = std::fs::read(path) else {
            return false;
        };
        if String::from_utf8_lossy(&bytes).contains("-shared") {
            return true;
        }
        let wide: Vec<_> = bytes
            .chunks_exact(2)
            .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
            .collect();
        String::from_utf16_lossy(&wide).contains("-shared")
    });
    if shared {
        command.args(["--entry", "DllMainCRTStartup"]);
    }
    match command.status() {
        Ok(status) if status.success() => ExitCode::SUCCESS,
        Ok(_) => ExitCode::FAILURE,
        Err(error) => {
            eprintln!("LLVM Windows linker failed: {error}");
            ExitCode::FAILURE
        }
    }
}
