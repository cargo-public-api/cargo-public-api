# public_items

List public items (the public API) of a library crate by analyzing rustdoc JSON of the crate.

# Usage

```bash
# Generate rustdoc JSON for your Rust library
RUSTDOCFLAGS='-Z unstable-options --output-format json' cargo +nightly doc --no-deps

# List all public items of the Rust library
cargo install public_items
public_items ./target/doc/your_library.json
```

# Target audience

Maintainers of Rust libraries that want to keep track of changes to their public API.

# Limitations

Currently:
* Only items from the crate itself are considered (so e.g. `fn clone()` will not be included in the output list, because the `Clone` trait is defined outside of the crate.)
* The type of items are not shown. So a struct field and and struct method is listed as `Struct::field` and `Struct::method`. And tuple structs will just be represented with `Struct::0`, `Struct::1`, etc. Since Rust does not support method overloading, this is not that big of an issue in practice.
