# Architecture

## Premise

The premise of this tool is the following: The best way to answer the question "What is the public API of my library crate?" is to run `cargo doc --open` and look for yourself.

The problem with rustdoc HTML output is that it does not lend itself well to API diffing. Luckily, a new rustdoc output format is being developed, namely [rustdoc JSON](https://github.com/rust-lang/rust/issues/76578).

This tool is based on rustdoc JSON.

## Listing

To list the public API of a crate, we first build rustdoc JSON for it, and then parse the rustdoc JSON. Building happens with the help of the [`rustdoc-json`](https://crates.io/crates/rustdoc-json) crate. All the JSON parsing and analysis is done in the [`public-api`](https://crates.io/crates/public-api) crate. The code for these crates are found in the corresponding directories. `cargo-public-api` is essentially a convenient wrapper on top of these low-level crates. But it does contain a fair amount of its own code. It needs to manipulate git repos and write syntax highlighted output, for example.

## Diffing

Diffing is essentially:
1. Build rustdoc JSON for two versions of a crate.
1. Independently parse rustdoc JSON for both versions via the `public-api` crate to get the full public API for each version.
1. Calculate the diff between the APIs (see [diff.rs](https://github.com/cargo-public-api/cargo-public-api/blob/main/public-api/src/diff.rs))
