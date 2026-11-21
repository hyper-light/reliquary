//! The recursion check's rustc wrapper: scripts/check-recursion.py compiles this file with the
//! pinned rustc and sets it as cargo's RUSTC_WORKSPACE_WRAPPER. It runs rustc with the arguments cargo
//! gives it, and a workspace library's or binary's compile also writes its LLVM IR beside its outputs,
//! in one codegen unit, so one .ll holds the crate and no stale unit's file is read with it. A program
//! rather than the script itself, which Windows cannot run as a wrapper; Mantle's check runs its
//! Python file as the wrapper (Mantle's scripts/check-recursion.py, `wrap`).

use std::ffi::OsString;
use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let mut args = std::env::args_os().skip(1);
    let Some(rustc) = args.next() else {
        eprintln!("recursion-wrapper: cargo gave no rustc to run");
        return ExitCode::from(2);
    };
    let args: Vec<OsString> = args.collect();
    let mut command = Command::new(rustc);
    command.args(&args);
    if writes_ir(&args) {
        command.args(["--emit=llvm-ir", "-C", "codegen-units=1"]);
    }
    match command.status() {
        Ok(status) => ExitCode::from(
            status
                .code()
                .and_then(|c| u8::try_from(c).ok())
                .unwrap_or(1),
        ),
        Err(error) => {
            eprintln!("recursion-wrapper: rustc didn't run: {error}");
            ExitCode::from(1)
        }
    }
}

/// Whether this compile is a library's or a binary's and not a build script's: rustc's `--crate-type`
/// and `--crate-name`, read from the arguments and from any `@file` cargo passes them in (one argument
/// a line, rustc's own format), as cargo does when a command line is too long for the OS.
fn writes_ir(args: &[OsString]) -> bool {
    let mut words: Vec<String> = Vec::new();
    for arg in args {
        let text = arg.to_string_lossy();
        match text.strip_prefix('@') {
            Some(path) => match std::fs::read_to_string(path) {
                Ok(file) => words.extend(file.lines().map(str::to_owned)),
                Err(_) => return false,
            },
            None => words.push(text.into_owned()),
        }
    }
    let after = |flag: &str| -> Vec<&str> {
        words
            .windows(2)
            .filter(|pair| pair.first().is_some_and(|w| w == flag))
            .filter_map(|pair| pair.get(1).map(String::as_str))
            .collect()
    };
    let types = after("--crate-type");
    let names = after("--crate-name");
    !types.is_empty()
        && types.iter().all(|t| *t == "lib" || *t == "bin")
        && names != ["build_script_build"]
}
