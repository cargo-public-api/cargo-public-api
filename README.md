# cargo-public-api

List and diff the public API of Rust library crates between releases and commits. Detect breaking API changes and semver violations via CI or a CLI. Relies on and automatically builds [rustdoc JSON](https://github.com/rust-lang/rust/issues/76578), for which a recent version of the Rust nightly toolchain must be installed.

# Installation

Install the `cargo public-api` subcommand with a recent regular **stable** Rust toolchain:

```sh
cargo +stable install cargo-public-api --locked
```

Ensure **nightly-2025-06-22** or later is installed (does not need to be the active toolchain) so `cargo public-api` can build rustdoc JSON for you:

```sh
rustup install nightly --profile minimal
```

# Usage

## List the Public API

This example lists the public API of the `regex` crate. First we clone the repo:

```sh
git clone https://github.com/rust-lang/regex ; cd regex
```

Now we can list the public API of `regex` by running

```sh
cargo public-api
```

which will print the public API of `regex` with one line per public item in the API:

<img src="https://github.com/cargo-public-api/cargo-public-api/raw/main/docs/img/list-truncated.webp" alt="colored output of listing a public api">

## Diff the Public API

### … Against a Specific Published Version

To diff the public API of the `regex` crate in the **current directory** against  **published version 1.6.0** on [crates.io](https://crates.io/crates/regex/1.6.0):

```sh
cargo public-api diff 1.6.0
```

<img src="https://github.com/cargo-public-api/cargo-public-api/raw/main/docs/img/diff-specific-published-version.webp" alt="colored output of diffing a public api">


### … Against the Latest Published Version

```sh
cargo public-api diff latest
```

### … Between Git Commits

```sh
cargo public-api diff ref1..ref2
```

### … as a CI Check

<!-- Keep this section in sync with ./public-api/README.md#public-api-surface-test-in-ci -->

With a regular `cargo test` that you run in CI you will be able to
* prevent accidental changes to your public API
* review the public API diff of deliberate changes [^1]

[^1]: As a workaround for https://github.com/mitsuhiko/insta/issues/780 you might want to put `*.snap linguist-language=txt` in your `.gitattributes`.

First add the latest versions of the recommended libraries to your `[dev-dependencies]`:

```sh
cargo add --dev \
    rustup-toolchain \
    rustdoc-json \
    public-api \
    insta
```

Then add the following test to your project. As the author of the below test code, I hereby associate it with [CC0](https://creativecommons.org/publicdomain/zero/1.0/) and to the extent possible under law waive all copyright and related or neighboring rights to it:

```rust
#[test]
fn public_api() {
    // Install a compatible nightly toolchain if it is missing
    rustup_toolchain::install(public_api::MINIMUM_NIGHTLY_RUST_VERSION).unwrap();

    // Build rustdoc JSON
    let rustdoc_json = rustdoc_json::Builder::default()
        .toolchain(public_api::MINIMUM_NIGHTLY_RUST_VERSION)
        .build()
        .unwrap();

    // Derive the public API from the rustdoc JSON
    let public_api = public_api::Builder::from_rustdoc_json(rustdoc_json)
        .build()
        .unwrap();

    // Assert that the public API looks correct
    insta::assert_snapshot!(public_api);
}
```

Before you run the test the first time you need to bless the current public API:

```sh
INSTA_UPDATE=always cargo test
```

This creates a `tests/snapshots/<module>_public_api.snap` file in your project that you `git add` together with your other project files. Then a regular

```sh
cargo test
```

will fail if your public API is accidentally or deliberately changed. Run

```sh
INSTA_UPDATE=always cargo test
```

again to review and accept public API changes.

## Less Noisy Output

For completeness, items belonging to _Blanket Implementations_, _Auto Trait Implementations_, and _Auto Derived Implementations_, such as

 * `impl<T, U> Into<U> for T where U: From<T>`
 * `impl Sync for ...`
 * `impl Debug for ...` / `#[derive(Debug)]`

are included in the list of public items by default. Use

 * `--omit blanket-impls`
 * `--omit auto-trait-impls`
 * `--omit auto-derived-impls`

respectively to omit such items from the output to make it much less noisy:

```sh
cargo public-api --omit blanket-impls,auto-trait-impls,auto-derived-impls
```

For convenience you can also use `-s` (`--simplified`) to achieve the same thing. This is a shorter form of the above command:

```sh
cargo public-api -sss
```

# Compatibility Matrix

| Version          | Understands the rustdoc JSON output of  |
| ---------------- | --------------------------------------- |
| 0.48.x           | nightly-2025-06-22 —                    |
| 0.47.x           | nightly-2025-03-24 — nightly-2025-06-21 |
| 0.46.x           | nightly-2025-03-16 — nightly-2025-03-23 |
| 0.45.x           | nightly-2025-03-14 — nightly-2025-03-15 |
| 0.43.x — 0.44.x  | nightly-2025-01-25 — nightly-2025-03-13 |
| 0.40.x — 0.42.x  | nightly-2024-10-18 — nightly-2025-01-24 |
| earlier versions | see [here](https://github.com/cargo-public-api/cargo-public-api/blob/main/scripts/release-helper/src/version_info.rs) |

# Contributing

See [CONTRIBUTING.md](./docs/CONTRIBUTING.md).

## Maintainers

- [Enselic](https://github.com/Enselic)
- [douweschulte](https://github.com/douweschulte)
- [Emilgardis](https://github.com/Emilgardis)

# Trademark Notice

"Rust" and "Cargo" are trademarks of the Rust Foundation. This project is not affiliated with, endorsed by, or otherwise associated with the Rust Project or Rust Foundation.
