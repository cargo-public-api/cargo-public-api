# cargo-public-items

List public items (the public API) of a Rust library crate by analyzing the rustdoc JSON of the crate.

Automatically builds the rustdoc JSON for you, which requires a nightly Rust toolchain to be installed.

## Installation

```
cargo install cargo-public-items
```

## Usage

To print all items that make up the public API of your Rust library crate, simply do this:

```
cd your-rust-library
cargo public-items
```

and the public API will be printed with one line per item:

```
your_rust_library
your_rust_library::some_function
your_rust_library::SomeStruct
your_rust_library::SomeStruct::some_struct_member
your_rust_library::SomeStruct::another_struct_member
```

Tip: If you pipe the output of different versions of your library to different files, you can use `diff` to diff the public API of your Rust library across versions.

# Target audience

Maintainers of Rust libraries that want to keep track of changes to their public API.

# Implementation details

This utility is a convenient `cargo` wrapper around the [public_items](https://crates.io/crates/public_items) crate (https://github.com/Enselic/public_items).
