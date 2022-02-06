# `cargo` wrapper for this library

You probably want the `cargo` wrapper to this library. See https://github.com/Enselic/cargo-public-items.

# public_items

List public items (the public API) of a Rust library crate by analyzing the rustdoc JSON of the crate.

# Usage

Again, you probably want to use the convenient [`cargo public-items`](https://crates.io/crates/cargo-public-items) wrapper. But if you don't want to use the `cargo` wrapper, you do as follows:

```bash
# Install the tool that comes with this package
cargo install public_items

# Generate rustdoc JSON for your Rust library
RUSTDOCFLAGS='-Z unstable-options --output-format json' cargo +nightly doc --lib --no-deps

# List all items in the public API of the Rust library using the tool
public_items ./target/doc/your_library.json
```

# Example

Using the tool on its own library:
```bash
% RUSTDOCFLAGS='-Z unstable-options --output-format json' cargo +nightly doc --lib --no-deps
% public_items ./target/doc/public_items.json
pub enum public_items::Error
pub enum variant public_items::Error::SerdeJsonError
pub fn public_items::Error::fmt(self, __formatter)
pub fn public_items::Error::fmt(self, f)
pub fn public_items::Error::from(source)
pub fn public_items::Error::source(self)
pub fn public_items::sorted_public_items_from_rustdoc_json_str(rustdoc_json_str)
pub mod public_items
pub struct field 0 (path missing due to https://github.com/rust-lang/rust/issues/92945)
pub type public_items::Result
```

Tip: By writing the public API to a file for two different versions of your library, you can diff your public API across versions.

# Target audience

Maintainers of Rust libraries that want to keep track of changes to their public API.

# Limitations

Currently:
* https://github.com/rust-lang/rust/issues/92945 causes issues for enum variant tuple struct fields.
* The full type of items are not included in the output.
* Items re-exported from other crates are not included in the output.

# Compatibility matrix

| public_items  | Understands the rustdoc JSON output of  |
| ------------- | --------------------------------------- |
| v0.0.6        | nightly-2022-01-19 —                    |
| v0.0.5        | nightly-2021-10-11 — nightly-2022-01-18 |
