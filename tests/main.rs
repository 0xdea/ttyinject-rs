//! Integration tests driving the compiled `ttyinject-rs` binary end-to-end to exercise `run`'s guard checks.

#![expect(clippy::expect_used, reason = "tests can use `expect`")]

/// Absolute path to the compiled `ttyinject-rs` binary under test.
const BIN: &str = env!("CARGO_BIN_EXE_ttyinject-rs");

#[cfg(test)]
mod tests {
    use std::os::fd::{FromRawFd as _, OwnedFd};
    use std::process::{Command, Stdio};
    use std::{io, ptr};

    use libc::{c_int, close, geteuid, openpty};

    use super::BIN;

    /// Asserts that `stderr` mentions `expected_message`, given verbose mode was enabled for the run.
    fn assert_stderr_contains(stderr: &str, expected_message: &str) {
        assert!(
            stderr.contains(expected_message),
            "expected stderr to contain {expected_message:?}, got: {stderr:?}"
        );
    }

    #[test]
    fn run_without_a_tty_fails() {
        let output = Command::new(BIN)
            .arg("-v")
            .stdin(Stdio::null())
            .output()
            .expect("failed to spawn the binary under test");

        assert!(
            !output.status.success(),
            "expected the binary to fail without a controlling tty"
        );

        let stderr = String::from_utf8_lossy(&output.stderr);
        assert_stderr_contains(&stderr, "stdin does not refer to a terminal");
    }

    #[test]
    fn run_with_our_own_tty_fails() {
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

        // SAFETY: `slave` is a valid, open pty fd from `openpty` above; ownership moves into `OwnedFd`, so it is
        // closed exactly once, when `Stdio::from` hands it to the child below.
        let slave_fd = unsafe { OwnedFd::from_raw_fd(slave) };

        // The child only needs the slave as its stdin: it never has to become this pty's session leader, because the
        // ownership check just stats whatever `/proc/self/fd/0` resolves to, and `is_terminal()` accepts any pty
        // slave.
        let output = Command::new(BIN)
            .arg("-v")
            .stdin(Stdio::from(slave_fd))
            .output()
            .expect("failed to spawn the binary under test");

        // SAFETY: `master` is a valid, open file descriptor from `openpty` above, closed exactly once here.
        unsafe {
            close(master);
        };

        assert!(
            !output.status.success(),
            "expected the binary to fail when its own tty has our uid"
        );

        let stderr = String::from_utf8_lossy(&output.stderr);
        // SAFETY: geteuid() is safe to call in this context
        if unsafe { geteuid() } == 0 {
            assert_stderr_contains(&stderr, "we are already root");
        } else {
            assert_stderr_contains(&stderr, "tty has the same uid as us");
        }
    }
}
