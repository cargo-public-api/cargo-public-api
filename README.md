# `cargo public-items` wrapper for this library

You might want the convenient `cargo public-items` wrapper for this library. See https://github.com/Enselic/cargo-public-items.

# public_items

List public items (the public API) of a Rust library crate by analyzing the rustdoc JSON of the crate. Enables diffing public API between releases.

# Usage

If you prefer not to use the convenient [`cargo public-items`](https://crates.io/crates/cargo-public-items) wrapper, do like this:

```bash
# Install the tool that is a thin wrapper around the `public_items` library
cargo install public_items

# Generate rustdoc JSON for your own Rust library
cd ~/src/your_library
RUSTDOCFLAGS='-Z unstable-options --output-format json' cargo +nightly doc --lib --no-deps

# List all items in the public API of your Rust library
public_items ./target/doc/your_library.json
```

# Example: List the public items of the `public_items` library itself

```txt
% git clone https://github.com/Enselic/public_items.git
% cd public_items
% RUSTDOCFLAGS='-Z unstable-options --output-format json' cargo +nightly doc --lib --no-deps
% public_items ./target/doc/public_items.json
pub enum public_items::Error
pub enum variant public_items::Error::SerdeJsonError(serde_json::Error)
pub fn public_items::Error::fmt(&self, __formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
pub fn public_items::Error::fmt(&self, f: &mut $crate::fmt::Formatter<'_>) -> $crate::fmt::Result
pub fn public_items::Error::from(source: serde_json::Error) -> Self
pub fn public_items::Error::source(&self) -> std::option::Option<&std::error::Error + 'static>
pub fn public_items::Options::clone(&self) -> Options
pub fn public_items::Options::default() -> Self
pub fn public_items::Options::fmt(&self, f: &mut $crate::fmt::Formatter<'_>) -> $crate::fmt::Result
pub fn public_items::PublicItem::cmp(&self, other: &PublicItem) -> $crate::cmp::Ordering
pub fn public_items::PublicItem::eq(&self, other: &PublicItem) -> bool
pub fn public_items::PublicItem::fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
pub fn public_items::PublicItem::ne(&self, other: &PublicItem) -> bool
pub fn public_items::PublicItem::partial_cmp(&self, other: &PublicItem) -> $crate::option::Option<$crate::cmp::Ordering>
pub fn public_items::sorted_public_items_from_rustdoc_json_str(rustdoc_json_str: &str, options: Options) -> Result<Vec<PublicItem>>
pub mod public_items
pub struct public_items::Options
pub struct public_items::PublicItem
pub struct field public_items::Options::with_blanket_implementations: bool
pub type public_items::Result<T> = std::result::Result<T, Error>
```

Tip: By writing the public API to a file for two different versions of your library, you can diff your public API across versions.

# Expected output

In general, output aims to be character-by-character identical to the textual parts of the regular `cargo doc` HTML output. For example, [this item](https://docs.rs/bat/0.20.0/bat/struct.PrettyPrinter.html#method.input_files) has the following textual representation in the rendered HTML:

```
pub fn input_files<I, P>(&mut self, paths: I) -> &mut Self
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
```

and `public_items` represent this item in the following manner:

```
pub fn bat::PrettyPrinter::input_files<I, P>(&mut self, paths: I) -> &mut Self where I: IntoIterator<Item = P>, P: AsRef<Path>
```

If normalize by removing newline characters and adding some whitespace padding to get the alignment right for direct comparison, we can see that they are exactly the same, except an irrelevant trailing comma:

```
pub fn                     input_files<I, P>(&mut self, paths: I) -> &mut Self where I: IntoIterator<Item = P>, P: AsRef<Path>,
pub fn bat::PrettyPrinter::input_files<I, P>(&mut self, paths: I) -> &mut Self where I: IntoIterator<Item = P>, P: AsRef<Path>
```

# Blanket implementations

By default, blanket implementations such as `impl<T> Any for T`, `impl<T> Borrow<T> for T`, and `impl<T, U> Into<U> for T where U: From<T>` are omitted from the list of public items of a crate. For the vast majority of use cases, blanket implementations are not of interest, and just creates noise.

If you want to include items of blanket implementations in the output, set [`Options::with_blanket_implementations`](https://docs.rs/public_items/latest/public_items/struct.Options.html#structfield.with_blanket_implementations) to true if you use the library, or pass `--with-blanket-implementations` to `public_items`.

# Target audience

Maintainers of Rust libraries that want to keep track of changes to their public API.

# Limitations

See [`[limitation]`](https://github.com/Enselic/public_items/labels/limitation)
labeled issues.

# Compatibility matrix

| public_items  | Understands the rustdoc JSON output of  |
| ------------- | --------------------------------------- |
| v0.4.x        | nightly-2022-02-23 —                    |
| v0.2.x        | nightly-2022-01-19 — nightly-2022-02-22 |
| v0.0.5        | nightly-2021-10-11 — nightly-2022-01-18 |
