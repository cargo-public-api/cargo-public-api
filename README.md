# `cargo` wrapper for this library

You probably want the `cargo` wrapper to this library. See https://github.com/Enselic/cargo-public-items.

# public_items

List public items (the public API) of a Rust library crate by analyzing the rustdoc JSON of the crate. Enables diffing public API between releases.

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
```txt
% RUSTDOCFLAGS='-Z unstable-options --output-format json' cargo +nightly doc --lib --no-deps
% public_items ./target/doc/public_items.json
pub enum public_items::Error
pub enum variant public_items::Error::SerdeJsonError(serde_json::Error)
pub fn public_items::Error::borrow(&self) -> &T
pub fn public_items::Error::borrow_mut(&mut self) -> &mut T
pub fn public_items::Error::fmt(&self, __formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
pub fn public_items::Error::fmt(&self, f: &mut $crate::fmt::Formatter<'_>) -> $crate::fmt::Result
pub fn public_items::Error::from(source: serde_json::Error) -> Self
pub fn public_items::Error::from(t: T) -> T
pub fn public_items::Error::into(self) -> U
pub fn public_items::Error::source(&self) -> std::option::Option<&std::error::Error + 'static>
pub fn public_items::Error::to_string(&self) -> String
pub fn public_items::Error::try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error>
pub fn public_items::Error::try_into(self) -> Result<U, <U as TryFrom<T>>::Error>
pub fn public_items::Error::type_id(&self) -> TypeId
pub fn public_items::PublicItem::borrow(&self) -> &T
pub fn public_items::PublicItem::borrow_mut(&mut self) -> &mut T
pub fn public_items::PublicItem::cmp(&self, other: &PublicItem) -> $crate::cmp::Ordering
pub fn public_items::PublicItem::eq(&self, other: &PublicItem) -> bool
pub fn public_items::PublicItem::fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
pub fn public_items::PublicItem::from(t: T) -> T
pub fn public_items::PublicItem::into(self) -> U
pub fn public_items::PublicItem::ne(&self, other: &PublicItem) -> bool
pub fn public_items::PublicItem::partial_cmp(&self, other: &PublicItem) -> $crate::option::Option<$crate::cmp::Ordering>
pub fn public_items::PublicItem::to_string(&self) -> String
pub fn public_items::PublicItem::try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error>
pub fn public_items::PublicItem::try_into(self) -> Result<U, <U as TryFrom<T>>::Error>
pub fn public_items::PublicItem::type_id(&self) -> TypeId
pub fn public_items::sorted_public_items_from_rustdoc_json_str(rustdoc_json_str: &str) -> Result<Vec<PublicItem>>
pub mod public_items
pub struct public_items::PublicItem
pub type public_items::Error::Error = <U as TryFrom<T>>::Error
pub type public_items::Error::Error = Infallible
pub type public_items::PublicItem::Error = <U as TryFrom<T>>::Error
pub type public_items::PublicItem::Error = Infallible
pub type public_items::Result<T> = std::result::Result<T, Error>
```

Tip: By writing the public API to a file for two different versions of your library, you can diff your public API across versions.

# Target audience

Maintainers of Rust libraries that want to keep track of changes to their public API.

# Limitations

See [`[limitation]`](https://github.com/Enselic/public_items/labels/limitation)
labeled issues.

# Compatibility matrix

| public_items  | Understands the rustdoc JSON output of  |
| ------------- | --------------------------------------- |
| v0.2.x        | nightly-2022-01-19 —                    |
| v0.0.5        | nightly-2021-10-11 — nightly-2022-01-18 |
