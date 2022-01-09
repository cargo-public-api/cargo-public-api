# public_items

List public items (the public API) of a Rust library crate by analyzing the rustdoc JSON of the crate.

# Usage

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
public_items
public_items::Error
public_items::Error::SerdeJsonError
public_items::Error::fmt
public_items::Error::from
public_items::Error::source
public_items::Result
public_items::from_rustdoc_json_str
```

Tip: By writing the public API to a file for two different versions of your library, you can diff your public API across versions.

# Target audience

Maintainers of Rust libraries that want to keep track of changes to their public API.

# Limitations

Currently:
* The type of items are not shown. So a struct field and and struct method is listed as `Struct::field` and `Struct::method`. And tuple structs will just be represented with `Struct::0`, `Struct::1`, etc. Since Rust does not support method overloading, this is not that big of an issue in practice.
