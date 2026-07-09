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
    // Handle verbose output with a macro.
    let verbose = env::args_os().nth(1).is_some();
    macro_rules! vprintln {
        ($($arg:tt)*) => {
            if verbose {
                eprintln!($($arg)*);
            }
        };
    }

    vprintln!("{PROGRAM} {VERSION} - Linux LPE via TIOCSTI tty injection");
    vprintln!("Copyright (c) 2026 {AUTHORS}");
    vprintln!();

    match ttyinject_rs::run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            vprintln!("[!] Error: {err:#}");
            ExitCode::FAILURE
        }
    }
}
