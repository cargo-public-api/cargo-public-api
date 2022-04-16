# cargo-public-api

List and diff the public API of Rust library crates between releases and commits. Allows you to detect breaking API changes and semver violations. Relies on and automatically builds rustdoc JSON, for which a recent version of the Rust nighty toolchain must be installed.

# Installation

```
# Install cargo-public-api with the regular stable Rust toolchain
cargo install cargo-public-api

# Install nightly-2022-03-14 or later so that rustdoc JSON can be built automatically by cargo-public-api
rustup install nightly
```

# Usage

## List public API

This example lists the public API of the library that this cargo subcommand is implemented with (before it was renamed to `public-api`). The example assumes that the git repository for the library is checked out to `~/src/public_items`. To print all items that make up the public API of your Rust library crate, simply do this:

```
% cd ~/src/public_items
% cargo public-api
```

and the public API will be printed with one line per item:

```
pub enum public_items::Error
pub enum variant public_items::Error::SerdeJsonError(serde_json::Error)
pub fn public_items::Error::fmt(&self, __formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
pub fn public_items::Error::fmt(&self, f: &mut $crate::fmt::Formatter<'_>) -> $crate::fmt::Result
pub fn public_items::Error::from(source: serde_json::Error) -> Self
pub fn public_items::Error::source(&self) -> std::option::Option<&std::error::Error + 'static>
pub fn public_items::PublicItem::cmp(&self, other: &PublicItem) -> $crate::cmp::Ordering
pub fn public_items::PublicItem::eq(&self, other: &PublicItem) -> bool
pub fn public_items::PublicItem::fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
pub fn public_items::PublicItem::ne(&self, other: &PublicItem) -> bool
pub fn public_items::PublicItem::partial_cmp(&self, other: &PublicItem) -> $crate::option::Option<$crate::cmp::Ordering>
pub fn public_items::sorted_public_items_from_rustdoc_json_str(rustdoc_json_str: &str) -> Result<Vec<PublicItem>>
pub mod public_items
pub struct public_items::PublicItem
pub type public_items::Result<T> = std::result::Result<T, Error>
```

## Diff public API between commits

To diff two different versions of your API, use `--diff-git-checkouts` while standing in the git repo of your project.

The following example prints the public API diff between v0.2.0 and v0.4.0 of the `public_items` library:

```
% cd ~/src/public_items
% cargo public-api --diff-git-checkouts v0.2.0 v0.4.0
## Removed items from the public API
(none)

## Changed items in the public API
* `pub fn public_items::sorted_public_items_from_rustdoc_json_str(rustdoc_json_str: &str) -> Result<Vec<PublicItem>>` changed to
  `pub fn public_items::sorted_public_items_from_rustdoc_json_str(rustdoc_json_str: &str, options: Options) -> Result<Vec<PublicItem>>`

## Added items to the public API
* `pub fn public_items::Options::clone(&self) -> Options`
* `pub fn public_items::Options::default() -> Self`
* `pub fn public_items::Options::fmt(&self, f: &mut $crate::fmt::Formatter<'_>) -> $crate::fmt::Result`
* `pub struct public_items::Options`
* `pub struct field public_items::Options::with_blanket_implementations: bool`
```

You can also manually do a diff by writing the full list of items to a file for two different versions of your library and then do a regular `diff` between the files.

# Target audience

Maintainers of Rust libraries that want to keep track of changes to their public API.

# Development tips

See [development.md](./doc/development.md).

# Implementation details

This utility is implemented with and adds conveniences on top of the [public-api](https://crates.io/crates/public-api) library (https://github.com/Enselic/public-api).
