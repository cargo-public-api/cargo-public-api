# cargo-public-api

List and diff the public API of Rust library crates between releases and commits. Detect breaking API changes and semver violations via CI or a CLI. Relies on and automatically builds [rustdoc JSON](https://github.com/rust-lang/rust/issues/76578), for which a recent version of the Rust nightly toolchain must be installed.

# Installation

Install the `cargo public-api` subcommand with a recent regular **stable** Rust toolchain:

```console
$ cargo install --locked cargo-public-api
```

Ensure **nightly-2023-01-04** or later is installed (does not need to be the active toolchain) so `cargo public-api` can build **rustdoc JSON** for you:

```console
$ rustup install --profile minimal nightly
```

# Usage

## List the Public API

This example lists the public API of the `regex` crate. First we clone the repo:

```console
$ git clone https://github.com/rust-lang/regex ; cd regex
```

Now we can list the public API of `regex` by running

```console
$ cargo public-api
```

which will print the public API of `regex` with one line per public item in the API:

<img src="docs/img/list-truncated.webp" alt="colored output of listing a public api">

## Diff the Public API

### … Against a Specific Published Version

To diff the public API of the `regex` crate in the **current directory** against  **published version 1.6.0** on [crates.io](https://crates.io/crates/regex/1.6.0):

```console
$ cargo public-api diff 1.6.0
```

<img src="docs/img/diff-specific-published-version.webp" alt="colored output of diffing a public api">


### … Against the Latest Published Version

```console
$ cargo public-api diff latest
Resolved `diff latest` to `diff 1.7.1`
[...]
```

### … Between Git Refs

```console
$ cargo public-api diff ref1..ref2
[...]
```

### … as a CI Check

With a regular `cargo test` that you run in CI you will be able to
* prevent accidental changes to your public API
* review the public API diff of deliberate changes

First add the latest versions of the necessary libraries to your `[dev-dependencies]`:

```console
$ cargo add --dev \
    rustup-toolchain \
    rustdoc-json \
    public-api \
    expect-test
```

Then add the following test to your project. As the author of the below test code I hereby put it in the public domain:

<!-- Keep this code in sync with the code in ./rustup-toolchain/tests/rustup-toolchain-lib-tests.rs -->
```rust
#[test]
fn public_api() {
    // Install a proper nightly toolchain if it is missing
    rustup_toolchain::ensure_installed(public_api::MINIMUM_NIGHTLY_VERSION).unwrap();

    // Build rustdoc JSON
    let rustdoc_json = rustdoc_json::Builder::default()
        .toolchain(public_api::MINIMUM_NIGHTLY_VERSION)
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

This creates a `tests/public-api.txt` file in your project that you `git add` together with your other project files. Whenever you change the public API, you need to bless it again with the above command. If you forget to bless, the test will fail, together with instructions on how to bless.

## Expected Output

Output aims to be character-by-character identical to the textual parts of the regular `cargo doc` HTML output. For example, [this item](https://docs.rs/bat/0.20.0/bat/struct.PrettyPrinter.html#method.input_files) has the following textual representation in the rendered HTML:

```
pub fn input_files<I, P>(&mut self, paths: I) -> &mut Self
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
```

and `cargo public-api` renders this item in the following way:

```
pub fn bat::PrettyPrinter::input_files<I, P>(&mut self, paths: I) -> &mut Self where I: IntoIterator<Item = P>, P: AsRef<Path>
```

If we normalize by removing newline characters and adding some whitespace padding to get the alignment right for side-by-side comparison, we can see that they are exactly the same, except an irrelevant trailing comma:

```
pub fn                     input_files<I, P>(&mut self, paths: I) -> &mut Self where I: IntoIterator<Item = P>, P: AsRef<Path>,
pub fn bat::PrettyPrinter::input_files<I, P>(&mut self, paths: I) -> &mut Self where I: IntoIterator<Item = P>, P: AsRef<Path>
```

## Less Noisy Output

For completeness, items belonging to _Blanket Implementations_ and _Auto Trait Implementations_ such as

* `impl<T> Borrow<T> for T`
* `impl<T, U> Into<U> for T where U: From<T>`
* `impl Send for ...`
* `impl Sync for ...`

are included in the list of public items by default. Use `--simplified` (or short form `-s`) to omit such items from the output:
```bash
cargo public-api --simplified
```

# Compatibility Matrix

| cargo-public-api | Understands the rustdoc JSON output of  |
| ---------------- | --------------------------------------- |
| 0.26.x — 0.27.x  | nightly-2023-01-04 —                    |
| 0.20.x — 0.25.x  | nightly-2022-09-28 — nightly-2023-01-03 |
| 0.19.x           | nightly-2022-09-08 — nightly-2022-09-27 |
| 0.18.x           | nightly-2022-09-07                      |
| 0.17.x           | nightly-2022-09-06                      |
| 0.15.x — 0.16.x  | nightly-2022-08-15 — nightly-2022-09-05 |
| 0.13.x — 0.14.x  | nightly-2022-08-10 — nightly-2022-08-14 |
| 0.12.x           | nightly-2022-05-19 — nightly-2022-08-09 |
| 0.10.x — 0.11.x  | nightly-2022-03-14 — nightly-2022-05-18 |
| earlier versions | see [here](https://github.com/Enselic/cargo-public-api/blob/0a37c971bb4ffa9bb11cf9fad9fef19aa67a986a/README.md#compatibility-matrix) |

# Contributing

See [CONTRIBUTING.md](./docs/CONTRIBUTING.md).

## Maintainers

- [Enselic](https://github.com/Enselic)
- [douweschulte](https://github.com/douweschulte)
- [Emilgardis](https://github.com/Emilgardis)
