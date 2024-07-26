# `public-api` changelog

## v0.37.0
* Move project from https://github.com/Enselic/cargo-public-api to https://github.com/cargo-public-api/cargo-public-api

## v0.36.0
* Render `pub const` types in more cases, such as for arrays and tuples.

## v0.35.1
* Don't panic when encountering constants without a type (e.g. `TakesConstGenericArg<120>`)

## v0.35.0
* Support `nightly-2024-06-07`.

## v0.33.1
* Fixup 'Avoid textual API diff when changing a trait impl to an auto-derived impl.'

## v0.33.0
* Avoid textual API diff when changing a trait impl to an auto-derived impl.

## v0.32.0
* Support `nightly-2023-08-25` and later
* Remove all deprecated API

## v0.31.0
* Change rendering of `impl` items to include generic args of the implementor
* Ignore `!` when sorting `impl`s to make `Send` and `Sync` order stable

## v0.30.0
* Support `nightly-2023-05-24` and later

## v0.29.1
* Prevent infinite RAM usage by hardening recursion detection

## v0.29.0
* Remove the `public-api` bin. Use `cargo public-api` instead.

## v0.28.0
* Rename `MINIMUM_NIGHTLY_VERSION` to `MINIMUM_NIGHTLY_RUST_VERSION` for clarity
* Deprecate `PublicApi::from_rustdoc_json()`. Use `public_api::Builder::from_rustdoc_json()` instead.
* Deprecate `Options`. Use `public_api::Builder` methods instead.

## v0.27.3
* Bump deps

## v0.27.2
* Bump deps

## v0.27.1
* Deprecate `PublicApi::from_rustdoc_json_str()`

## v0.27.0
* Make `Eq` and `Hash` for `PublicItem` only take tokens into account, to make diffing insensitive to internal data that has no bearing on public API surface of a crate
* Remove `Ord` and `PartialOrd` impls of `PublicItem` and `ChangedPublicItem`. Use `PublicItem::grouping_cmp` and `ChangedPublicItem::grouping_cmp` instead.
* Rename `MINIMUM_RUSTDOC_JSON_VERSION` to `MINIMUM_NIGHTLY_VERSION` for clarity

## v0.26.0
* Put auto-derived impls (`Clone`, `Debug`, etc) in a separate group, right after normal `impl`s
* Remove deprecated `Options::with_blanket_implementations`
* Split `Options::simplified` into `Options::omit_auto_trait_impls` and `Options::omit_blanket_impls`

## v0.25.0
* Get rid of `enum variant` and `struct field` prefixes in rendered items
* Group impl blocks together with their respective functions
* Bump all deps

## v0.24.1
* Make `PublicApi` implement `Display` to get `.to_string()`. All items in one big multi-line `String`.

## v0.22.0
* Remove deprecated `fn public_api_from_rustdoc_json_str()`

## v0.21.0
* Rename `PublicItemsDiff` to `PublicApiDiff`
* Make `PublicApiDiff::between(...)` take `PublicApi`s instead of `Vec<PublicItem>`s
* Hide `PublicApi::items` behind `PublicApi::items()` iterator
* Add `PublicApi::from_rustdoc_json(...)` that takes a rustdoc JSON file `Path`

## v0.20.1
* Allow to specify `--target-dir` when building rustdoc JSON

## v0.20.0
* deprecate `public_api_from_rustdoc_json_str` and replace with method on `PublicApi`
* move `rustdoc_json::build` into a struct fn `BuildOptions::build`

## v0.15.0
* Introduce `PublicApi` struct

## v0.12.4
* Add `PublicItemsDiff::is_empty()`

## v0.12.0
* Add `Token::Annotation`

## v0.11.5
* impl `Hash` for `PublicItem`

## v0.10.0
* Remove `TokenStream`

## v0.9.0
* Rename project from `public_items` to `public-api` and add `Token`s
