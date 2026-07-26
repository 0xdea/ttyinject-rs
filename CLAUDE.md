# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

A Rust port of [hackerschoice/ttyinject](https://github.com/hackerschoice/ttyinject): a Linux local-privilege-escalation
proof-of-concept that abuses the `TIOCSTI` ioctl to inject keystrokes into a root shell's tty input buffer when root
runs `su - user`. It is a single-purpose security research tool, not a general-purpose library — the README's only
planned addition is CLI arguments for a custom command/line count.

## Linux-only

`src/lib.rs` has `#[cfg(not(target_os = "linux"))] compile_error!(...)` at the top. The crate **cannot be built on
macOS/Windows** — `cargo build`/`cargo test` will fail immediately with that compile error on non-Linux hosts. This is
expected, not a bug to fix.

## Commands

```sh
cargo build --locked                          # CI does NOT let Cargo touch the lockfile
cargo fmt --all --check                        # formatting check (CI runs this, not `cargo fmt --all`)
cargo clippy --all-targets --locked -- -D warnings
cargo doc --no-deps --locked                   # RUSTDOCFLAGS=-D warnings in CI
cargo test --locked -- --nocapture
cargo test <test_name>                         # run a single test by name (substring match)
cargo audit                                    # requires cargo-audit installed
cargo semver-checks                            # requires cargo-semver-checks installed
```

`taplo` formats `Cargo.toml` per `.taplo.toml` (120-column width, 4-space indent).

## Lints

`Cargo.toml` turns on `clippy::pedantic`, `clippy::nursery`, `clippy::cargo`, and `clippy::restriction` (all four at
`warn`), plus `missing_docs = warn`, under `[workspace.lints]`. A long allowlist under `[workspace.lints.clippy]`
carves out specific lints from `restriction` that don't fit this codebase (e.g. `implicit_return`,
`print_stdout`/`print_stderr`, `question_mark_used`, `std_instead_of_core`). When adding code, match the existing
style rather than suppressing new lints locally — check that allowlist first if clippy complains about something
that looks like a style choice rather than a bug.

Unsafe blocks throughout `src/lib.rs` and the tests carry `// SAFETY:` comments justifying each invariant. Preserve
this convention for any new `unsafe` code.

## Architecture

- **`src/main.rs`** — thin CLI shell. Enables verbose `eprintln!` output (via the `vprintln!` macro) if any argument is
  passed, calls `ttyinject_rs::run()`, and on success clears 4 terminal lines via `ttyinject_rs::clear_terminal`. All
  actual logic lives in the library so it's independently testable.
- **`src/lib.rs`** — the exploit, as a sequence of guarded steps in `run()`:
  1. `check_preconditions` — stdin must be a tty, euid must not already be 0, parent pid must be valid (order matters:
     tty check runs before the root check — see the `check_preconditions_checks_tty_before_root` test).
  2. `check_tty_ownership` — the controlling tty must not already be owned by our own uid (i.e. there must actually be
     a different, more-privileged user's shell attached to it).
  3. Self-delete the running executable (`fs::remove_file` on `env::current_exe()`) so it only ever runs once.
  4. Inject a single test byte via `tiocsti_inject` to confirm `TIOCSTI` actually works before proceeding.
  5. `SIGSTOP` the parent process so it stops consuming input, `sleep(100ms)`, then inject the real payload
     (`START` + `COMMAND` + `END` byte constants) one byte at a time via `ioctl(fd, TIOCSTI, &byte)`. The payload
     copies `/bin/sh` to `/var/tmp/.socket`, `chmod`s it setuid, and ends with `fg`, which resumes (`SIGCONT`s) the
     parent itself — there's no explicit `SIGCONT` call in the code.
- `tiocsti_inject` only succeeds against a tty that is the _calling process's own controlling terminal_ (unless the
  caller has `CAP_SYS_ADMIN`), and only if the `dev.tty.legacy_tiocsti` sysctl allows it (disabled by default since
  Linux 6.2). Both the unit test (`tiocsti_inject_via_pty`, in `src/lib.rs`) and integration tests treat ioctl failure
  from this restriction as an expected outcome, not a test failure — don't "fix" that by making the test assert
  success.
- **`tests/main.rs`** — black-box integration tests that spawn the _compiled binary_ (`env!("CARGO_BIN_EXE_ttyinject-rs")`)
  with controlled stdin (`Stdio::null()`, or a PTY slave fd) and assert on exit status and stderr content. These
  complement the unit tests in `src/lib.rs`, which test the guard functions directly.
- `[profile.release]` is tuned for a small, standalone dropped binary: `strip`, `lto`, `opt-level = "z"`,
  `codegen-units = 1`, `panic = "abort"`.

## CI

- **`.github/workflows/build.yml`** — runs on every push/PR to `master`: rustfmt check, `cargo build --locked`,
  clippy, `cargo-audit`, doc build, tests, `cargo-semver-checks`, plus a separate `zizmor` job that lints the GitHub
  Actions workflows themselves for security issues.
- **`.github/workflows/release.yml`** — triggered by `v*` tags. Cross-compiles static musl binaries for a matrix of
  Linux targets (see the commented-out target list for what's available but currently disabled) via `cross`, then
  publishes them as GitHub release assets. Update `CHANGELOG.md` and bump the version in `Cargo.toml` before tagging.
