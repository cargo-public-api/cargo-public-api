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
