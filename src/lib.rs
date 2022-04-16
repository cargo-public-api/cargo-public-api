//! This library gives you a list of all public items (otherwise known as the
//! public API) of a library crate. As input to the library, a special output
//! format from `cargo doc` is used, which goes by the name **rustdoc JSON**.
//! Currently, only `cargo doc` from the Nightly toolchain can produce **rustdoc
//! JSON** for a library. You build **rustdoc JSON** like this:
//!
//! ```bash
//! RUSTDOCFLAGS='-Z unstable-options --output-format json' cargo +nightly doc --lib --no-deps
//! ```
//!
//! The main entry point to the library is
//! [`public_items_from_rustdoc_json_str`], so please read its documentation.
//!
//! # Examples
//!
//! The two main use cases are listing public items, and diffing public items.
//!
//! ## List all public items of a crate (the public API)
//! ```
#![doc = include_str!("../examples/list_public_items.rs")]
//! ```
//!
//! ## Diff two versions of a public API
//! ```
#![doc = include_str!("../examples/diff_public_items.rs")]
//! ```
//!
//! The most comprehensive example code on how to use the library can be found
//! in the thin binary wrapper around the library, see
//! <https://github.com/Enselic/public_items/blob/main/src/main.rs>.

#![deny(missing_docs)]
#![deny(clippy::all, clippy::pedantic)]

mod error;
mod intermediate_public_item;
mod item_iterator;
mod render;
pub mod tokens;

pub mod diff;

// Documented at the definition site so cargo doc picks it up
pub use error::{Error, Result};

pub use item_iterator::PublicItem;

/// This constant defines the minimum version of nightly that is required in
/// order for the rustdoc JSON output to be parsable by this library. Note that
/// this library is implemented with stable Rust. But the rustdoc JSON that this
/// library parses can currently only be produced by nightly.
///
/// The rustdoc JSON format is still changing, so every now and then we update
/// this library to support the latest format. If you use this version of
/// nightly or later, you should be fine.
pub const MINIMUM_RUSTDOC_JSON_VERSION: &str = "nightly-2022-03-14";

/// Contains various options that you can pass to [`public_items_from_rustdoc_json_str`].
#[derive(Copy, Clone, Debug)]
#[non_exhaustive] // More options are likely to be added in the future
pub struct Options {
    /// If `true`, items part of blanket implementations such as `impl<T> Any
    /// for T`, `impl<T> Borrow<T> for T`, and `impl<T, U> Into<U> for T where
    /// U: From<T>` are included in the list of public items of a crate.
    ///
    /// The default value is `false` since the the vast majority of users will
    /// find the presence of these items to just constitute noise, even if they
    /// formally are part of the public API of a crate.
    pub with_blanket_implementations: bool,

    /// If `true`, items will be sorted before being returned. If you will pass
    /// on the return value to [`diff::PublicItemsDiff::between`], it is
    /// currently unnecessary to sort first, because the return value will
    /// internally be converted into a `HashSet`.
    ///
    /// The default value is `true`, because usually the performance impact is
    /// negligible, and is is generally more practical to work with sorted data.
    pub sorted: bool,
}

/// Enables options to be set up like this (note that `Options` is marked
/// `#[non_exhaustive]`):
///
/// ```
/// # use public_items::Options;
/// let mut options = Options::default();
/// options.sorted = true;
/// // ...
/// ```
impl Default for Options {
    fn default() -> Self {
        Self {
            with_blanket_implementations: false,
            sorted: true,
        }
    }
}

/// Takes rustdoc JSON and returns a [`Vec`] of [`PublicItem`]s where each
/// [`PublicItem`] is one public item of the crate, i.e. part of the crate's
/// public API.
///
/// There exists a convenient `cargo public-api` subcommand wrapper for this
/// function found at <https://github.com/Enselic/cargo-public-api> that
/// builds the rustdoc JSON for you and then invokes this function. If you don't
/// want to use that wrapper, use
/// ```bash
/// RUSTDOCFLAGS='-Z unstable-options --output-format json' cargo +nightly doc --lib --no-deps
/// ```
/// to generate the rustdoc JSON that this function takes as input. The output
/// is put in `./target/doc/your_library.json`.
///
/// For reference, the rustdoc JSON format is documented at
/// <https://rust-lang.github.io/rfcs/2963-rustdoc-json.html>. But the format is
/// still a moving target. Open PRs and issues for rustdoc JSON itself can be
/// found at <https://github.com/rust-lang/rust/labels/A-rustdoc-json>.
///
/// # Errors
///
/// E.g. if the JSON is invalid.
pub fn public_items_from_rustdoc_json_str(
    rustdoc_json_str: &str,
    options: Options,
) -> Result<Vec<PublicItem>> {
    let crate_: rustdoc_types::Crate = serde_json::from_str(rustdoc_json_str)?;

    let mut public_items: Vec<_> = item_iterator::public_items_in_crate(&crate_, options).collect();

    if options.sorted {
        public_items.sort();
    }

    Ok(public_items)
}
