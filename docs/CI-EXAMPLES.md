# CI Examples

This document describes different ways to make use of `cargo public-api` in CI. For regular usage, see [Usage](../README.md#usage) instead.

## Prevent Accidental Public API Changes

### With Blessable `cargo test`

The best way to version the public API of your crate - and require any changes to show up in diffs - is to write a regular `cargo test` that you run via CI along with all other tests.

First add the latest versions of the necessary libraries to your `[dev-dependencies]`:

```console
$ cargo add --dev \
    rustup-toolchain \
    rustdoc-json \
    public-api \
    expect-test
```

Then copy-paste this test to your project:

```rust
#[test]
fn public_api() {
    // Install a proper nightly toolchain if it is missing
    rustup_toolchain::ensure_installed(public_api::MINIMUM_RUSTDOC_JSON_VERSION).unwrap();

    // Build rustdoc JSON
    let rustdoc_json = rustdoc_json::Builder::default()
        .toolchain(public_api::MINIMUM_RUSTDOC_JSON_VERSION.to_owned())
        .build()
        .unwrap();

    // Derive the public API from the rustdoc JSON
    let public_api =
        public_api::PublicApi::from_rustdoc_json(rustdoc_json, public_api::Options::default())
            .unwrap();

    // Assert that the public API looks correct
    expect_test::expect_file!["public-api.txt"].assert_eq(&public_api.to_string());
}
```

Before you run the test the first time you need to bless the current public API:

```console
$ UPDATE_EXPECT=1 cargo test public_api
```

Whenever you change the public API, you need to bless it again with the above command.

This create a `tests/public-api.txt` file in your project that you version together with your other project files. Any changes to it (and the public API) will show up in e.g. PR diffs.

### Locking

Since the rustdoc JSON format is unstable and frequently changes across nightly toolchain versions, and since improvements to `cargo public-api` are regularly released, you probably want to lock against a specific `0.y.z` version of `public-api` and a specific version of the nightly toolchain. The above example code does that. To find other matching versions, consult the [Compatibility Matrix](../README.md#compatibility-matrix).
