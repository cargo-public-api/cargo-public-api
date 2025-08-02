# rustdoc-json

## v0.9.7
* Bump `cargo_metadata` from `0.19.2` to `0.21.0`.

## v0.9.6
* Don't panic on `resolver = 3` in `Cargo.toml`.

## v0.9.5
* Insignificant maintenance release (fix a typo and remove a couple of `dev-dependencies` due to a moved test).

## v0.9.4
* Bump deps. Most notably `cargo-manifest` from 0.16.0 to 0.17.0.

## v0.9.3
* Bump deps. Most notably `cargo-manifest` from 0.15.0 to 0.16.0.

## v0.9.2
* Move project from https://github.com/Enselic/cargo-public-api to https://github.com/cargo-public-api/cargo-public-api

## v0.9.1
* Add `rustdoc_json::Builder::build_with_captured_output(self, stdout: impl std::io::Write, stderr: impl std::io::Write)`.
* Introduce new errors `rustdoc_json::BuildError::BuildRustdocJsonError`, `BuildError::CapturedOutputError` and `BuildError::CommandExecutionError` and partially switch to those from `BuildError::General`.
* Add `rustdoc_json::Builder::color(self, color: rustdoc_json::Color)` to control `--color` of `cargo`.

## v0.9.0
* Remove `rustdoc_json::Builder::verbose()` and use the `tracing` crate for debug logging instead.
* Print a nice error message if `rustup` is not in `PATH`.
* Bump cargo-manifest from v0.13.0 to 0.14.0.

## v0.8.9
* Bump deps. Most notably cargo-manifest from 0.12.0 to 0.13.0.

## v0.8.8
* Add `rustdoc_json::Builder::verbose(bool)`

## v0.8.7
* Handle when `[package]` `name` differs from `[lib]` `name`

## v0.8.6
* Replace `-` with `_` also in e.g. bin packages
* Remove support for `"crate_name@1.2.3"` to `Builder::package_target()`. I don't think anyone uses that. Let me know if you do and if I need to yank this release and bump to v0.9.0.

## v0.8.5
* Simplify `BuildError::General` error message
* Bump deps

## v0.8.4
* Make `rustdoc_json::Builder::clear_toolchain()` actually clear the toolchain

## v0.8.3
* Bump deps

## v0.8.2
* Bump deps, most notably `toml` from 0.5.11 to 0.7.2

## v0.8.1
* Bump deps

## v0.8.0
* Change `Builder::toolchain(...)` to take `Into<String>` instead of `Into<Option<String>>` to make client code nicer in 99% of cases. Introduce `Builder::clear_toolchain()` for the 1%.

## v0.7.4
* Correctly determine json path for `Builder::default().package("crate@1.0.0")`
* Add `Builder::package_target()` and `PackageTarget`
* Add `Builder::silent()` to suppress stdout and stderr
* Bump all deps

## v0.7.3
* Derive `Clone` for `rustdoc_json::Builder`

## v0.7.2
* Add `Builder::document_private_items()`

## v0.7.1
* Add `Builder::clear_target_dir()`

## v0.7.0
* Remove deprecated `BuildOptions` and `fn build(...)`. Use `Builder` and `Builder::build()` instead.
* Use `cargo-manifest` to parse Cargo manifests

## v0.6.0
* Remove `BuildError::CargoTomlError` (and `cargo_toml` dependency)

## v0.5.0
* Remove most of `BuildOptions`, only leave deprecation message

## v0.4.2
* Add `Builder::target_dir()`

## v0.4.1
* rename `BuildOptions` to `Builder`, a new insta-deprecated alias `BuildOptions` is available
* deprecate `build()`, replaced with `Builder::build()`
* Allow changing `--cap-lints`

## v0.4.0
* Support for specifying `--target`, `--features`, and `--package`
* Make it clearer that `RUSTUP_TOOLCHAIN` and friends has an impact

## v0.3.1
* Don't eat up stdout and stderr

## v0.3.0
* Change `fn build()` to take `BuildOptions`

## v0.2.2
* Make sure `RUSTDOC` and `RUSTC` env vars are cleared before building rustdoc JSON

## v0.2.1
* First public release
