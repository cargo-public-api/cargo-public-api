# You probably want the CLI

This is a low level Rust library. You probably want to use this high level and convenient CLI: [`cargo public-api`](https://github.com/Enselic/cargo-public-api).

# public-api

List and diff the public API of Rust library crates by analyzing rustdoc JSON output files from the nightly toolchain.

This library is the backbone of [`cargo public-api`](https://github.com/Enselic/cargo-public-api).

# Usage

The library comes with a thin bin wrapper that can be used to explore the capabilities of this library.

```bash
# Build and install the thin bin wrapper with a recent stable Rust toolchain
cargo install public-api

# Install nightly-2022-08-10 or later so you can build up-to-date rustdoc JSON files
rustup install nightly
```

## List the public API

To list all items that form the public API of your Rust library:

```bash
# Generate rustdoc JSON for your own Rust library
% cd ~/src/your_library
% cargo +nightly rustdoc --lib -- -Z unstable-options --output-format json

# List all items in the public API of your Rust library
% public-api ./target/doc/your_library.json
pub mod public_api
pub fn public_api::Options::clone(&self) -> Options
pub fn public_api::Options::default() -> Self
pub fn public_api::PublicItem::clone(&self) -> PublicItem
pub fn public_api::public_api_from_rustdoc_json_str(rustdoc_json_str: &str, options: Options) -> Result<Vec<PublicItem>>
pub struct public_api::Options
pub struct public_api::PublicItem
pub struct field public_api::Options::sorted: bool
pub struct field public_api::Options::with_blanket_implementations: bool
...
```

## Diff the public API

It is frequently of interest to know how the public API of a crate has changed. You can find this out by doing a diff between different versions of the same library. The higher level tool  [`cargo public-api`](https://github.com/Enselic/cargo-public-api/tree/main/cargo-public-api) makes this more convenient, but it is possible without it.

```bash
# Generate two different rustdoc JSON files for two different versions of your library
# and then pass both files to the bin to make it print the public API diff
% public-api ./target/doc/your_library.old.json  ./target/doc/your_library.json
Removed:
(nothing)

Changed:
-pub fn public_api::sorted_public_api_from_rustdoc_json_str(rustdoc_json_str: &str) -> Result<Vec<PublicItem>>
+pub fn public_api::sorted_public_api_from_rustdoc_json_str(rustdoc_json_str: &str, options: Options) -> Result<Vec<PublicItem>>

Added:
+pub fn public_api::Options::clone(&self) -> Options
+pub fn public_api::Options::default() -> Self
+pub struct public_api::Options
+pub struct field public_api::Options::with_blanket_implementations: bool
```

# Library documentation

Documentation can be found at [docs.rs](https://docs.rs/public-api/latest/public-api/) as usual. There are also some simple [examples](https://github.com/Enselic/cargo-public-api/tree/main/public-api/examples) on how to use the library. The code for the [thin bin wrapper](https://github.com/Enselic/cargo-public-api/blob/main/public-api/src/main.rs) might also be of interest.

# Historical note

The code for this library used to live at https://github.com/Enselic/public-api. So git tags for old versions are found in that repo rather than this repo.
