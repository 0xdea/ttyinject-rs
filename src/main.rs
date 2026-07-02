//! main.rs.

// Standard library imports.
use std::env;
use std::ffi::OsString;
use std::path::Path;
use std::process::ExitCode;

// External crate imports.

// Internal crate imports.

// const NAME: type = ...;

// static NAME: type = ...;

/// Package name.
const PROGRAM: &str = env!("CARGO_PKG_NAME");
/// Package version.
const VERSION: &str = env!("CARGO_PKG_VERSION");
/// Package authors.
const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");

fn main() -> ExitCode {
    eprintln!("{PROGRAM} {VERSION} - Linux LPE via TIOCSTI TTY injection");
    eprintln!("Copyright (c) 2026 {AUTHORS}");
    eprintln!();

    /*
    // Parse command line arguments.
    let mut args = env::args_os();
    let argv0 = args.next().unwrap_or_else(|| OsString::from(PROGRAM));
    // let is_help = |a: &OsStr| a == OsStr::new("-h") || a == OsStr::new("--help");

    let prog = Path::new(&argv0)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(PROGRAM);

    let action = match (args.next(), args.next()) {
        (None, _) => OsString::from("default"),
        (Some(arg), None) if !is_help(&arg) => arg,
        _ => return usage(prog),
    };
    */

    // Let's do it.
    match ttyinject_rs::run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("[!] Error: {err:#}");
            ExitCode::FAILURE
        }
    }
}

/// Prints usage information and exits.
fn usage(prog: &str) -> ExitCode {
    eprintln!("Usage:");
    eprintln!("{prog} TODO");
    eprintln!("\nExamples:");
    eprintln!("{prog}");
    eprintln!("{prog} TODO");

    ExitCode::FAILURE
}
