# `cargo-public-api` changelog

## v0.47.1
* Don't panic on `resolver = 3` in `Cargo.toml`.

## v0.47.0
* Support `nightly-2025-03-24` and later.
* Restore old and correct `#[repr(...)]` rendering

## v0.46.0
* Placeholder (not released at the moment).

## v0.45.0
* Placeholder (not released at the moment).

## v0.44.2
* Support `nightly-2025-02-26` and later but use a quick-hack where `#[repr(...)]` is rendered as e.g. `#[attr="Repr([ReprInt(UnsignedInt(U8))])")]`.

## v0.44.1
* Support precise capturing syntax in function return types: `-> impl Sized + use<'a, T>`

## v0.44.0
* Do not render function arg names `""` (`nightly-2025-02-05` and later)

## v0.43.0
* Support `nightly-2025-01-25` and later.

## v0.42.0
* Render `_` argument names in function declarations

## v0.41.0
* Render `?` in front of `core::marker::Sized` if applicable.

## v0.40.0
* Support `nightly-2024-10-18` and later

## v0.39.0
* Support `nightly-2024-10-13` and later

## v0.38.0
* Support `nightly-2024-09-10` and later

## v0.37.0
* Support `nightly-2024-07-05`
* Properly render lifetime bounds
* Move project from https://github.com/Enselic/cargo-public-api to https://github.com/cargo-public-api/cargo-public-api

## v0.36.0
* Render `pub const` types in more cases, such as for arrays and tuples

## v0.35.1
* Don't panic when encountering constants without a type

## v0.35.0
* Support `nightly-2024-06-07` and later

## v0.34.2
* Allow `stdout` and `stderr` of rustdoc JSON building to be captured with new `rustdoc_json::Builder::build_with_captured_output(self, stdout: impl Write, stderr: impl Write)` function
* Make most CLI options global so they also work as subcommand args--package=...`
* Match `cargo`'s handling of spaces with `--features`

## v0.34.1
* Make diffing against published crate work when its `[package] name` is not the same as its `[lib] name`
* Bump cargo-manifest from v0.13.0 to v0.14.0
* Use the `tracing` crate for debug logging and e.g. `RUST_LOG=debug` to activate

## v0.34.0
* Remove `cargo public-api --toolchain foo` arg. Use `cargo +foo public-api` instead.
* Include all subcommands in top-level `--help` output
* Print a nice error message if `rustup` is not in `PATH`
* Update `cargo-manifest` from `0.12.0` to `0.13.0`

## v0.33.1
* Fixup 'Avoid textual API diff when changing a trait impl to an auto-derived impl'

## v0.33.0
* Avoid textual API diff when changing inherent impl to auto-derived impl
* Add relevant `package.keywords` to our manifests

## v0.32.0
* Remove $ from README commands to ease copy-paste
* Support for `nightly-2023-08-25` and later

## v0.31.3
* Handle when `[package]` `name` differs from `[lib]` `name`

## v0.31.2
* Support sparse crates.io registry index

## v0.31.1
* Print more informative error when we encounter a sparse crates.io index

## v0.31.0
* Change rendering of `impl` items to include generic args of the implementor
* Ignore `!` when sorting `impl`s to make `Send` and `Sync` order stable
* Rustdoc-JSON: Replace `-` with `_` for binary package target

## v0.30.0
* Support `nightly-2023-05-24` and later

## v0.29.1
* Prevent infinite RAM usage

## v0.29.0
* Interpret `cargo public-api diff` as `cargo public-api diff latest`
* Remove the `public-api` bin

## v0.28.0
* Enable `,` as `--omit` value delimiter
* Change `-s` / `--simplified` to be usable up to 3 times
* Introduce `public_api::Builder`, deprecate `PublicApi::from_rustdoc_json()` and `Options`
* public-api: Rename `MINIMUM_NIGHTLY_VERSION` to `MINIMUM_NIGHTLY_RUST_VERSION`
* rustup-toolchain: Rename `ensure_installed()` to `install()` and deprecate `is_installed()`

## v0.27.3
* Support installing the `cargo-public-api` bin with a different name
* Add explicit variants of `--simplified` called `--omit blanket-impls | auto-trait-impls | auto-derived-impls`
* Add `cargo public-api completions <SHELL>` to generate shell completion scripts

## v0.27.2
* Support for selecting features with `cargo public-api diff x.y.z`

## v0.27.1
* Fix RUSTSEC-2023-0003 / CVE-2023-22742 "git2 does not verify SSH keys
* Make `diff latest` diff highest semver version, not most recently published
* When diffing, build rustdoc JSON with `--cap-lints allow`
* Deprecate `PublicApi::from_rustdoc_json_str()`

## v0.27.0
* Group `impl`s better, make sorting more lexicographic
* Properly group multiple inherent `impl`s
### Other Changes
* README: Fix typo in compatibility matrix

## v0.26.0
* If `--simplified` is passed twice, omit auto derived `impl`s
* Support `cargo public-api diff latest` to diff against the latest published version
* Fix rendering of bounds for Generic Associated Types
* Remove deprecated `--diff` CLI
* Remove `diff crate-name@0.1.0` support, use `-p crate-name diff 0.1.0` instead
* Put auto derived `impl`s
* Bump minimum nightly version to `nightly-2023-01-04`

## v0.25.0
* Group impl blocks together with their respective functions
* Get rid of `enum variant` and `struct field` prefixes
* Upgrade all deps
* Add `rustdoc_json::Builder::package_target()` to build for bin, test, bench, etc
* Correctly determine JSON path for `Builder::package("foo@1.0.0")`

## v0.24.2
* Interpret `--color` as `--color=always`
* Deprecate legacy `--diff` CLI and point to new `diff` subcommand instead
* Support `--document-private-items`
* Derive `Clone` for `rustdoc_json::Builder`

## v0.24.1
* Add new CLI for diffing: a `diff` subcommand

## v0.24.0
* Include Blanket and Auto Trait `impl`s
* Allow omitting package name when diffing against crates.io

For older releases, see https://github.com/cargo-public-api/cargo-public-api/releases/tag/v0.23.0 and earlier.
