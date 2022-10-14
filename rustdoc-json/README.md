# rustdoc-json

Utilities for working with [rustdoc JSON](https://github.com/rust-lang/rust/issues/76578). Please refer to the [`docs`](https://docs.rs/rustdoc-json) for more info and example code.

Originally developed for use by [`public-api`](https://crates.io/crates/public-api) and [`cargo-public-api`](https://crates.io/crates/cargo-public-api), but should be useful for any Rust code that wants to build rustdoc JSON.

## Changelog

Please refer to [CHANGELOG.md](CHANGELOG.md).

## Testing

This library is indirectly tested through the [`public-api`](https://crates.io/crates/public-api) and [`cargo-public-api`](https://crates.io/crates/cargo-public-api) test suites. Their tests heavily depend on this library, so if all of their tests pass, then this library works as it should. All tests are of course ensured to pass before a new release is made.
