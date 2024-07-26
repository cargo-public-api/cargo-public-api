# rustdoc-json

A library for programmatically working with [rustdoc JSON](https://github.com/rust-lang/rust/issues/76578).

## Build rustdoc JSON

To build rustdoc JSON for a library with the manifest path `project/Cargo.toml`, do like this:

```rust
let json_path = rustdoc_json::Builder::default()
    .toolchain("nightly")
    .manifest_path("project/Cargo.toml")
    .build()
    .unwrap();

// Prints `Wrote rustdoc JSON to "/Users/martin/src/project/target/doc/project.json"`
println!("Wrote rustdoc JSON to {:?}", &json_path);
```

There are many more build options. See the [docs](https://docs.rs/rustdoc-json/latest/rustdoc_json/struct.Builder.html) to learn about all of them.

## Changelog

Please refer to [CHANGELOG.md](https://github.com/cargo-public-api/cargo-public-api/blob/main/rustdoc-json/CHANGELOG.md).

## Tests

This library is indirectly and heavily tested through the [`public-api`](https://crates.io/crates/public-api) and [`cargo-public-api`](https://crates.io/crates/cargo-public-api) test suites. Their tests heavily depend on this library, so if all of their tests pass, then this library works as it should. All tests are of course ensured to pass before a new release is made.
