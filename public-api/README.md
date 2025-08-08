# public-api

List and diff the public API of Rust library crates by analyzing rustdoc JSON output files from `rustdoc +nightly`.

# Usage

## … as a Rust library

See [docs.rs](https://docs.rs/public-api/latest/public_api/index.html) for library documentation and example code.

## … as a CLI

Use [`cargo public-api`](https://github.com/cargo-public-api/cargo-public-api) for CLI use cases.

## … as a CI Check

<!-- Keep this section in sync with the ./README.md#-as-a-ci-check -->

With a regular `cargo test` that you run in CI you will be able to
* prevent accidental changes to your public API
* review the public API diff of deliberate changes

First add the latest versions of the recommended libraries to your `[dev-dependencies]`:

```sh
cargo add --dev \
    rustup-toolchain \
    rustdoc-json \
    public-api
```

Then add the following test to your project. As the author of the below test code, I hereby associate it with [CC0](https://creativecommons.org/publicdomain/zero/1.0/) and to the extent possible under law waive all copyright and related or neighboring rights to it:

```rust
#[test]
fn public_api() {
    // Install a compatible nightly toolchain if it is missing.
    rustup_toolchain::install(public_api::MINIMUM_NIGHTLY_RUST_VERSION).unwrap();

    // Build rustdoc JSON.
    let rustdoc_json = rustdoc_json::Builder::default()
        .toolchain(public_api::MINIMUM_NIGHTLY_RUST_VERSION)
        .build()
        .unwrap();

    // Derive the public API from rustdoc JSON.
    let public_api = public_api::Builder::from_rustdoc_json(rustdoc_json)
        .build()
        .unwrap();

    // Assert that the public API matches the latest snapshot.
    // Run with env var `UPDATE_SNAPSHOTS=yes` to update.
    public_api.assert_eq_or_update("./tests/snapshots/public-api.txt");
}
```

Before you run the test the first time you need create a snapshot of the current public API:

```sh
UPDATE_SNAPSHOTS=yes cargo test
```

This creates a `tests/public-api.txt` file in your project that you `git add` together with your other project files. Then a regular

```sh
cargo test
```

will fail if your public API is accidentally or deliberately changed. Run

```sh
UPDATE_SNAPSHOTS=yes cargo test
```

again to update the public API snapshot and review the git diff.

# Maintainers

See [here](https://github.com/cargo-public-api/cargo-public-api#maintainers).
