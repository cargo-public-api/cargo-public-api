# `cargo public-items` wrapper for this library

You might want the convenient `cargo public-items` wrapper for this library. See https://github.com/Enselic/cargo-public-items.

# public_items

List public items (the public API) of Rust library crates by analyzing their rustdoc JSON. Also supports diffing the public API between releases and commits to e.g. help find breaking API changes or semver violations.

# Usage

The library comes with a thin bin wrapper that can be used to explore the capabilities of this library.

```bash
# Install the thin bin wrapper
% cargo install public_items

# Install a recent version of nightly so that rustdoc JSON can be generated
% rustup install nightly
```

## List public items

To list all items that form the public API of your Rust library:

```bash
# Generate rustdoc JSON for your own Rust library
% cd ~/src/your_library
% RUSTDOCFLAGS='-Z unstable-options --output-format json' cargo +nightly doc --lib --no-deps

# List all items in the public API of your Rust library
% public_items ./target/doc/your_library.json
pub mod public_items
pub fn public_items::Options::clone(&self) -> Options
pub fn public_items::Options::default() -> Self
pub fn public_items::PublicItem::clone(&self) -> PublicItem
pub fn public_items::public_items_from_rustdoc_json_str(rustdoc_json_str: &str, options: Options) -> Result<Vec<PublicItem>>
pub struct public_items::Options
pub struct public_items::PublicItem
pub struct field public_items::Options::sorted: bool
pub struct field public_items::Options::with_blanket_implementations: bool
...
```

## Diff public items

It is frequently of interest to know how the public API of a crate has changed. You can find this out by doing a diff between different versions of the same library. Again, [`cargo public-items`](https://github.com/Enselic/cargo-public-items) makes this more convenient, but it is straightforward enough without it.

```bash
# Generate two different rustdoc JSON files for two different versions of your library
# and then pass both files to the bin to make it print the public API diff
% public_items ./target/doc/your_library.old.json  ./target/doc/your_library.json
Removed:
(nothing)

Changed:
-pub fn public_items::sorted_public_items_from_rustdoc_json_str(rustdoc_json_str: &str) -> Result<Vec<PublicItem>>
+pub fn public_items::sorted_public_items_from_rustdoc_json_str(rustdoc_json_str: &str, options: Options) -> Result<Vec<PublicItem>>

Added:
+pub fn public_items::Options::clone(&self) -> Options
+pub fn public_items::Options::default() -> Self
+pub struct public_items::Options
+pub struct field public_items::Options::with_blanket_implementations: bool
```

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

If we normalize by removing newline characters and adding some whitespace padding to get the alignment right for side-by-side comparison, we can see that they are exactly the same, except an irrelevant trailing comma:

```
pub fn                     input_files<I, P>(&mut self, paths: I) -> &mut Self where I: IntoIterator<Item = P>, P: AsRef<Path>,
pub fn bat::PrettyPrinter::input_files<I, P>(&mut self, paths: I) -> &mut Self where I: IntoIterator<Item = P>, P: AsRef<Path>
```

# Blanket implementations

By default, blanket implementations such as `impl<T> Any for T`, `impl<T> Borrow<T> for T`, and `impl<T, U> Into<U> for T where U: From<T>` are omitted from the list of public items of a crate. For the vast majority of use cases, blanket implementations are not of interest, and just creates noise.

If you want to include items of blanket implementations in the output, set [`Options::with_blanket_implementations`](https://docs.rs/public_items/latest/public_items/struct.Options.html#structfield.with_blanket_implementations) to true if you use the library, or pass `--with-blanket-implementations` to `public_items`.

# Library documentation

Documentation can be found at [docs.rs](https://docs.rs/public_items/latest/public_items/) as usual. There are also some simple [examples](https://github.com/Enselic/public_items/tree/main/examples) on how to use the library. The code for the [thin bin wrapper](https://github.com/Enselic/public_items/blob/main/src/main.rs) might also be of interest.

# Target audience

Maintainers of Rust libraries that want to keep track of changes to their public API.

# Limitations

See [`[limitation]`](https://github.com/Enselic/public_items/labels/limitation)
labeled issues.

# Compatibility matrix

| public_items  | Understands the rustdoc JSON output of  |
| ------------- | --------------------------------------- |
| v0.7.x        | nightly-2022-03-14 —                    |
| v0.5.x        | nightly-2022-02-23 — nightly-2022-03-13 |
| v0.2.x        | nightly-2022-01-19 — nightly-2022-02-22 |
| v0.0.5        | nightly-2021-10-11 — nightly-2022-01-18 |
