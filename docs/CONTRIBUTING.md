# Architecture

Before you get started, you might want to read about the [architecture](./ARCHITECTURE.md).

# Getting started

Just clone the repo. Then you can make changes and run tests:

```
git clone https://github.com/cargo-public-api/cargo-public-api.git ; cd cargo-public-api

cargo test
```

This project makes heavy use of CI. To simulate the CI pipeline locally, run
```
./scripts/run-ci-locally.sh
```

Note that you can run `./scripts/run-ci-locally.sh` from from within your IDE. Then you can simply click on errors to navigate to the proper file, line number, and column. See `.vscode/tasks.json` for an example configuration.

## Blessing new expected output

To make your changes become the expected output, run
```
./scripts/bless-expected-output-for-tests.sh
```

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

If we remove newline characters and add some whitespace padding to get the alignment right for side-by-side comparison, we can see that they are exactly the same, except an irrelevant trailing comma:

```
pub fn                     input_files<I, P>(&mut self, paths: I) -> &mut Self where I: IntoIterator<Item = P>, P: AsRef<Path>,
pub fn bat::PrettyPrinter::input_files<I, P>(&mut self, paths: I) -> &mut Self where I: IntoIterator<Item = P>, P: AsRef<Path>
```

# Constraints

## Minimum required stable Rust version

This project is guaranteed to build with the latest stable Rust toolchain. More specifically, the toolchain that is installed by default on GitHub's `ubuntu-latest` runner. You can see [here](https://github.com/actions/runner-images/blob/main/images/ubuntu/Ubuntu2204-Readme.md#rust-tools) what version that currently is.

Note that the toolchain required to build this project is distinct from the toolchain required to build the rustdoc JSON that this project relies on. See below for more info.

## Minimum required nightly Rust version

Since the rustdoc JSON format still changes in incompatible ways, there is a lower bound on what nightly version you can use. For regular users, that minimal nightly version is mentioned in the README.md. For developers however, a more recent version can be needed. This is because even though the rustdoc JSON format is unchanged, its output can change. See [this PR](https://github.com/cargo-public-api/cargo-public-api/pull/84) for just one example.

Our CI runs every night, so any problems are generally detected quickly. If `cargo test` fails, make sure you have a recent enough nightly toolchain installed.

# Tips to work on this tool

## Run local copy of `cargo-public-api` on an arbitrary crate

There are two ways. You can either do:
```
% cd ~/src/arbitrary-crate
% cargo run --manifest-path ~/src/cargo-public-api/cargo-public-api/Cargo.toml
```
or you can do
```
% cd ~/src/cargo-public-api
% cargo run --bin cargo-public-api -- --manifest-path ~/src/arbitrary-crate/Cargo.toml
```
In the first case `--manifest-path` is interpreted by `cargo` itself, and in the second case `--manifest-path` is interpreted by `cargo-public-api`.

You can also combine both ways:
```
% cd /does/not/matter
% cargo run --manifest-path ~/src/cargo-public-api/cargo-public-api/Cargo.toml -- --manifest-path ~/src/arbitrary-crate/Cargo.toml
```

## Use custom rustdoc JSON toolchain

If you have built rustdoc yourself to try some rustdoc JSON fix, you can run `cargo public-api` with your [custom toolchain](https://rustc-dev-guide.rust-lang.org/building/how-to-build-and-run.html#creating-a-rustup-toolchain) like this:

```
cargo +custom public-api
```

Another option is the `RUSTDOC_JSON_OVERRIDDEN_TOOLCHAIN_HACK` env var. Use it like this:
```
RUSTDOC_JSON_OVERRIDDEN_TOOLCHAIN_HACK=custom ./scripts/run-ci-locally.sh
```

# Automated tests

All features and bugfixes needs automated tests. The only way to make sure no regressions creep in in software that is constantly changed, is to test for it. But manually testing quickly becomes unmanageable. Therefore, automated tests are needed.

# Maintainer guidelines

Please see [MAINTAINER.md](./MAINTAINER.md).
