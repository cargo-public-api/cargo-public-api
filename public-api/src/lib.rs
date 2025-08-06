//! This library gives you the public API of a library crate, in the form of a
//! list of public items in the crate. Public items are items that other crates
//! can use. Diffing is also supported.
//!
//! If you want a convenient CLI for this library, you should use [cargo
//! public-api](https://github.com/cargo-public-api/cargo-public-api).
//!
//! As input to the library, a special output format from `rustdoc +nightly` is
//! used, which goes by the name **rustdoc JSON**. Currently, only the nightly
//! toolchain can build **rustdoc JSON**.
//!
//! You use the [`rustdoc-json`](https://crates.io/crates/rustdoc_json) library
//! to programmatically build rustdoc JSON. See below for example code. To
//! manually build rustdoc JSON you would typically do something like this:
//! ```sh
//! cargo +nightly rustdoc -- -Z unstable-options --output-format json
//! ```
//!
//! # Examples
//!
//! The two main use cases are listing the public API and diffing different
//! versions of the same public APIs.
//!
//! ## List all public items of a crate (the public API)
//! ```no_run
#![doc = include_str!("../examples/list_public_api.rs")]
//! ```
//!
//! ## Diff two versions of a public API
//! ```no_run
#![doc = include_str!("../examples/diff_public_api.rs")]
//! ```

// deny in CI, only warn here
#![warn(clippy::all, missing_docs)]

mod crate_wrapper;
mod error;
mod intermediate_public_item;
mod item_processor;
mod nameable_item;
mod path_component;
mod public_item;
mod render;
pub mod tokens;

pub mod diff;

use std::path::PathBuf;

// Documented at the definition site so cargo doc picks it up
pub use error::{Error, Result};

// Documented at the definition site so cargo doc picks it up
pub use public_item::PublicItem;

/// This constant defines the minimum version of nightly that is required in
/// order for the rustdoc JSON output to be parsable by this library. Note that
/// this library is implemented with stable Rust. But the rustdoc JSON that this
/// library parses can currently only be produced by nightly.
///
/// The rustdoc JSON format is still changing, so every now and then we update
/// this library to support the latest format. If you use this version of
/// nightly or later, you should be fine.
pub const MINIMUM_NIGHTLY_RUST_VERSION: &str = "nightly-2025-08-02";
// End-marker for scripts/release-helper/src/bin/update-version-info/main.rs

/// See [`Builder`] method docs for what each field means.
#[derive(Copy, Clone, Debug)]
struct BuilderOptions {
    sorted: bool,
    debug_sorting: bool,
    omit_blanket_impls: bool,
    omit_auto_trait_impls: bool,
    omit_auto_derived_impls: bool,
}

/// Builds [`PublicApi`]s. See the [top level][`crate`] module docs for example
/// code.
#[derive(Debug, Clone)]
pub struct Builder {
    rustdoc_json: PathBuf,
    options: BuilderOptions,
}

impl Builder {
    /// Create a new [`PublicApi`] builder from a rustdoc JSON file. See the
    /// [top level][`crate`] module docs for example code.
    #[must_use]
    pub fn from_rustdoc_json(path: impl Into<PathBuf>) -> Self {
        let options = BuilderOptions {
            sorted: true,
            debug_sorting: false,
            omit_blanket_impls: false,
            omit_auto_trait_impls: false,
            omit_auto_derived_impls: false,
        };
        Self {
            rustdoc_json: path.into(),
            options,
        }
    }

    /// If `true`, items will be sorted before being returned. If you will pass
    /// on the return value to [`diff::PublicApiDiff::between`], it is
    /// currently unnecessary to sort first, because the sorting will be
    /// performed/ensured inside of that function.
    ///
    /// The default value is `true`, because usually the performance impact is
    /// negligible, and is is generally more practical to work with sorted data.
    #[must_use]
    pub fn sorted(mut self, sorted: bool) -> Self {
        self.options.sorted = sorted;
        self
    }

    /// If `true`, item paths include the so called "sorting prefix" that makes
    /// them grouped in a nice way. Only intended for debugging this library.
    ///
    /// The default value is `false`
    #[must_use]
    pub fn debug_sorting(mut self, debug_sorting: bool) -> Self {
        self.options.debug_sorting = debug_sorting;
        self
    }

    /// If `true`, items that belongs to Blanket Implementations are omitted
    /// from the output. This makes the output less noisy, at the cost of not
    /// fully describing the public API.
    ///
    /// Examples of Blanket Implementations: `impl<T> Any for T`, `impl<T>
    /// Borrow<T> for T`, and `impl<T, U> Into<U> for T where U: From<T>`
    ///
    /// The default value is `false` so that the listed public API is complete
    /// by default.
    #[must_use]
    pub fn omit_blanket_impls(mut self, omit_blanket_impls: bool) -> Self {
        self.options.omit_blanket_impls = omit_blanket_impls;
        self
    }

    /// If `true`, items that belongs to Auto Trait Implementations are omitted
    /// from the output. This makes the output less noisy, at the cost of not
    /// fully describing the public API.
    ///
    /// Examples of Auto Trait Implementations: `impl Send for Foo`, `impl Sync
    /// for Foo`, and `impl Unpin for Foo`
    ///
    /// The default value is `false` so that the listed public API is complete
    /// by default.
    #[must_use]
    pub fn omit_auto_trait_impls(mut self, omit_auto_trait_impls: bool) -> Self {
        self.options.omit_auto_trait_impls = omit_auto_trait_impls;
        self
    }

    /// If `true`, items that belongs to automatically derived implementations
    /// (`Clone`, `Debug`, `Eq`, etc) are omitted from the output. This makes
    /// the output less noisy, at the cost of not fully describing the public
    /// API.
    ///
    /// The default value is `false` so that the listed public API is complete
    /// by default.
    #[must_use]
    pub fn omit_auto_derived_impls(mut self, omit_auto_derived_impls: bool) -> Self {
        self.options.omit_auto_derived_impls = omit_auto_derived_impls;
        self
    }

    /// Builds [`PublicApi`]. See the [top level][`crate`] module docs for
    /// example code.
    ///
    /// # Errors
    ///
    /// E.g. if the [JSON](Builder::from_rustdoc_json) is invalid or if the file
    /// can't be read.
    pub fn build(self) -> Result<PublicApi> {
        from_rustdoc_json_str(std::fs::read_to_string(self.rustdoc_json)?, self.options)
    }
}

/// The public API of a crate
///
/// Create an instance with [`Builder`].
///
/// ## Rendering the items
///
/// To render the items in the public API you can iterate over the [items](PublicItem).
///
/// You get the `rustdoc_json_str` in the example below as explained in the [crate] documentation, either via
/// [`rustdoc_json`](https://crates.io/crates/rustdoc_json) or by calling `cargo rustdoc` yourself.
///
/// ```no_run
/// use public_api::PublicApi;
/// use std::path::PathBuf;
///
/// # let rustdoc_json: PathBuf = todo!();
/// // Gather the rustdoc content as described in this crates top-level documentation.
/// let public_api = public_api::Builder::from_rustdoc_json(&rustdoc_json).build()?;
///
/// for public_item in public_api.items() {
///     // here we print the items to stdout, we could also write to a string or a file.
///     println!("{}", public_item);
/// }
///
/// // If you want all items of the public API in a single big multi-line String then
/// // you can do like this:
/// let public_api_string = public_api.to_string();
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug)]
#[non_exhaustive] // More fields might be added in the future
pub struct PublicApi {
    /// The items that constitutes the public API. An "item" is for example a
    /// function, a struct, a struct field, an enum, an enum variant, a module,
    /// etc...
    pub(crate) items: Vec<PublicItem>,

    /// See [`Self::missing_item_ids()`]
    pub(crate) missing_item_ids: Vec<u32>,
}

impl PublicApi {
    /// Returns an iterator over all public items in the public API
    pub fn items(&self) -> impl Iterator<Item = &'_ PublicItem> {
        self.items.iter()
    }

    /// Like [`Self::items()`], but ownership of all `PublicItem`s are
    /// transferred to the caller.
    pub fn into_items(self) -> impl Iterator<Item = PublicItem> {
        self.items.into_iter()
    }

    /// The rustdoc JSON IDs of missing but referenced items. Intended for use
    /// with `--verbose` flags or similar.
    ///
    /// In some cases, a public item might be referenced from another public
    /// item (e.g. a `mod`), but is missing from the rustdoc JSON file. This
    /// occurs for example in the case of re-exports of external modules (see
    /// <https://github.com/cargo-public-api/cargo-public-api/issues/103>). The entries
    /// in this Vec are what IDs that could not be found.
    ///
    /// The exact format of IDs are to be considered an implementation detail
    /// and must not be be relied on.
    pub fn missing_item_ids(&self) -> impl Iterator<Item = &u32> {
        self.missing_item_ids.iter()
    }

    /// Assert that the public API matches the text-file snapshot at
    /// `snapshot_path`. The function will panic after printing a helpful diff
    /// if the public API does not match.
    ///
    /// If the env var `UPDATE_SNAPSHOTS` is set to `1`, `yes`, or `true`, the
    /// public API will be written to the snapshot file instead of being
    /// asserted to match.
    #[cfg(feature = "snapshot-testing")]
    pub fn assert_eq_or_update(&self, snapshot_path: impl AsRef<std::path::Path>) {
        let update = std::env::var("UPDATE_SNAPSHOTS")
            .map_or(false, |s| s == "1" || s == "yes" || s == "true");

        if update {
            std::fs::write(&snapshot_path, self.to_string())
                .unwrap_or_else(|e| panic!("Error writing `{:?}`: {e}", snapshot_path.as_ref()));
        } else {
            let expected = std::fs::read_to_string(&snapshot_path)
                .unwrap_or_else(|e| panic!("Error reading `{:?}`: {e}", snapshot_path.as_ref()));
            similar_asserts::assert_eq!(self.to_string(), expected);
        }
    }
}

impl std::fmt::Display for PublicApi {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for item in self.items() {
            writeln!(f, "{item}")?;
        }
        Ok(())
    }
}

fn from_rustdoc_json_str(
    rustdoc_json_str: impl AsRef<str>,
    options: BuilderOptions,
) -> Result<PublicApi> {
    let crate_ = deserialize_without_recursion_limit(rustdoc_json_str.as_ref())?;

    let mut public_api = item_processor::public_api_in_crate(&crate_, options);

    if options.sorted {
        public_api.items.sort_by(PublicItem::grouping_cmp);
    }

    Ok(public_api)
}

/// Helper to deserialize the JSON with `serde_json`, but with the recursion
/// limit disabled. Otherwise we hit the recursion limit on crates such as
/// `diesel`.
fn deserialize_without_recursion_limit(rustdoc_json_str: &str) -> Result<rustdoc_types::Crate> {
    let mut deserializer = serde_json::Deserializer::from_str(rustdoc_json_str);
    deserializer.disable_recursion_limit();
    Ok(serde::de::Deserialize::deserialize(&mut deserializer)?)
}
