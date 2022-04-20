# cargo-public-api

List and diff the public API of Rust library crates between releases and commits. Allows you to detect breaking API changes and semver violations. Relies on and automatically builds rustdoc JSON, for which a recent version of the Rust nighty toolchain must be installed.

# Installation

```
# Install cargo-public-api with the regular stable Rust toolchain
cargo install cargo-public-api

# Install nightly-2022-03-14 or later so cargo-public-api can build rustdoc JSON for you
rustup install nightly
```

# Usage

## List public API

This example lists the public API of the ubiquitous `regex` crate. First let's clone the repo:

```
% git clone https://github.com/rust-lang/regex ~/src/regex
```

Now we can list the public API of `regex` by doing

```
% cd ~/src/regex
% cargo public-api
```

which will print the public API of `regex` with one line per public item in the API:

```
pub enum regex::Error
pub enum variant regex::Error::CompiledTooBig(usize)
pub enum variant regex::Error::Syntax(String)
pub fn regex::CaptureLocations::clone(&self) -> CaptureLocations
pub fn regex::CaptureLocations::fmt(&self, f: &mut $crate::fmt::Formatter<'_>) -> $crate::fmt::Result
pub fn regex::CaptureLocations::get(&self, i: usize) -> Option<(usize, usize)>
pub fn regex::CaptureLocations::len(&self) -> usize
pub fn regex::CaptureMatches::fmt(&self, f: &mut $crate::fmt::Formatter<'_>) -> $crate::fmt::Result
pub fn regex::CaptureMatches::next(&mut self) -> Option<Captures<'t>>
...
```

## Diff public API

To diff the API between say **1.3.0** and **1.4.0** of `regex`, use `--diff-git-checkouts` while standing in the git repo. Like this:

```
% cd ~/src/regex
% cargo public-api --diff-git-checkouts 1.3.0 1.4.0
```

and the API diff will be printed:

```
Removed items from the public API
=================================
(none)

Changed items in the public API
===============================
(none)

Added items to the public API
=============================
pub fn regex::Match::range(&self) -> Range<usize>
pub fn regex::RegexSet::empty() -> RegexSet
pub fn regex::RegexSet::is_empty(&self) -> bool
pub fn regex::SubCaptureMatches::clone(&self) -> SubCaptureMatches<'c, 't>
pub fn regex::bytes::Match::range(&self) -> Range<usize>
pub fn regex::bytes::RegexSet::empty() -> RegexSet
pub fn regex::bytes::RegexSet::is_empty(&self) -> bool
pub fn regex::bytes::SubCaptureMatches::clone(&self) -> SubCaptureMatches<'c, 't>
```

You can also manually do a diff by writing the full list of items to a file for two different versions of your library and then do a regular `diff` between the files.

# Output formats

Currently there are two output formats. You can choose between `--output-format plain` (default) and `--output-format markdown`.

# Target audience

Maintainers of Rust libraries that want to keep track of changes to their public API.

# Implementation details

This utility is implemented with and adds conveniences on top of the [public-api](https://crates.io/crates/public-api) library (https://github.com/Enselic/public-api).

# Development tips

See [development.md](./doc/development.md).

# Maintainers

- [Enselic](https://github.com/Enselic)
- [douweschulte](https://github.com/douweschulte)
