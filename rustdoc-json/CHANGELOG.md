## Unreleased v0.8.0
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
