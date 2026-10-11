//! rustc's dlltool protocol using its bundled LLVM import-library writer.
//! The GNU dlltool shipped by this compiler requires an assembler it does not bundle.
use std::{env, ffi::OsString, process::{Command, ExitCode}};
fn main() -> ExitCode {
    let args: Vec<OsString> = env::args_os().skip(1).collect();
    let value = |flag: &str| args.windows(2).find(|pair| pair[0] == flag).map(|pair| pair[1].clone());
    let (Some(def), Some(output), Some(machine)) = (value("-d"), value("-l"), value("-m")) else {
        eprintln!("Windows import writer requires -d DEFINITION -l OUTPUT -m MACHINE");
        return ExitCode::FAILURE;
    };
    if machine != "i386:x86-64" { eprintln!("Only Windows x64 import libraries are supported"); return ExitCode::FAILURE; }
    let Some(linker) = env::var_os("OPSEC_RUST_LLD") else {eprintln!("OPSEC_RUST_LLD is unset");return ExitCode::FAILURE;};
    let prefixed = |prefix: &str, path: OsString| {let mut arg=OsString::from(prefix);arg.push(path);arg};
    match Command::new(linker).args([OsString::from("-flavor"),OsString::from("link"),OsString::from("/lib"),OsString::from("/machine:x64"),prefixed("/def:",def),prefixed("/out:",output)]).status() {
        Ok(status) if status.success() => ExitCode::SUCCESS,
        Ok(_) => ExitCode::FAILURE,
        Err(error) => {eprintln!("LLVM import-library writer failed: {error}");ExitCode::FAILURE}
    }
}
