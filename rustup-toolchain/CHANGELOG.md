# rustup-toolchain

If a version is not listed below, it means it had no API changes.

## v0.1.4
* Add `rustup_toolchain::Installer` to make the API more future-proof and configurable. For starters, the `--profile` can now be chosen by clients.
* Deprecate `rustup_toolchain::ensure_installed(...)`. Use `rustup_toolchain::Installer::default().toolchain(...).run()` instead.

## v0.1.3
* Bump all deps

## v0.1.2
* Bump all deps

## v0.1.1
* Bump all deps

## v0.1.0
* First public release
