# Changelog for ttyinject-rs

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.2] - 2026-11-26

### Changed

- Replace `anyhow::bail!` with `anyhow::ensure!` where possible.
- Use the `--locked` flag for all suitable `cargo` commands.
- Improve code style.
- Improve documentation.
- Improve CI.
- Align lint allowlist with my other projects.
- Update dependencies.
- Enable Cargo lints and fail CI on their warnings.
- Allow `unwrap`, `expect`, `panic!`, and indexing in tests via `clippy.toml`.
- Add a docs.rs documentation link to `Cargo.toml` and a doc badge to the README.

### Fixed

- Fall back to uncached builds in CI when sccache fails to start.

## [0.1.1] - 2026-07-19

### Changed

- Improve `Cargo.toml` formatting.
- Improve documentation.

## [0.1.0] - 2026-07-19

- First release to be published to [crates.io](https://crates.io/).

[unreleased]: https://github.com/0xdea/ttyinject-rs/compare/v0.1.2...HEAD
[0.1.2]: https://github.com/0xdea/ttyinject-rs/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/0xdea/ttyinject-rs/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/0xdea/ttyinject-rs/releases/tag/v0.1.0
