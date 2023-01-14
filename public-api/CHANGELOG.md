# public-api

If a version is not listed below, it means it had no API changes.

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
