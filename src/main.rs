//! main.rs.

use std::env;
use std::process::ExitCode;

/// Package name.
const PROGRAM: &str = env!("CARGO_PKG_NAME");
/// Package version.
const VERSION: &str = env!("CARGO_PKG_VERSION");
/// Package authors.
const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");

fn main() -> ExitCode {
    eprintln!("{PROGRAM} {VERSION} - Linux LPE via TIOCSTI tty injection");
    eprintln!("Copyright (c) 2026 {AUTHORS}");
    eprintln!();

    // Let's do it.
    match ttyinject_rs::run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("[!] Error: {err:#}");
            ExitCode::FAILURE
        }
    }
}
