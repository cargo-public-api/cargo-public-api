mod error;
mod implementation;

// Documented at the definition site so cargo doc picks it up
pub use error::Error;

// Documented at the definition site so cargo doc picks it up
pub use error::Result;

/// Represent a public item of an analyzed crate, i.e. an item that forms part
/// of the public API of a crate. Implements `Display` so it can be printed. It
/// also implements [`Ord`], but how items are ordered are not stable yet, and
/// will change in later versions.
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct PublicItem {
    /// Private implementation detail. The "pub struct/fn/..." part of an item.
    prefix: String,

    /// Private implementation detail. The "your_crate::mod_a::mod_b" part of an
    /// item.
    path: String,

    /// Private implementation detail. The type info part, e.g. "(param_a: Type,
    /// param_b: OtherType)" for a `fn`.
    suffix: String,
}

/// Contains various options that you can pass to
/// [`sorted_public_items_from_rustdoc_json_str`].
#[derive(Copy, Clone, Debug)]
#[non_exhaustive]
pub struct Options {
    /// If `true`, items part of blanket implementations such as `impl<T> Any
    /// for T`, `impl<T> Borrow<T> for T`, and `impl<T, U> Into<U> for T where
    /// U: From<T>` are included in the list of public items of a crate.
    ///
    /// The default value is `false` since the the vast majority of users will
    /// find the presence of these items to just constitute noise, even if they
    /// formally are part of the public API of a crate.
    pub with_blanket_implementations: bool,
}

/// Implementation that allows passing `Options::default()` for the `options`
/// parameter to [`sorted_public_items_from_rustdoc_json_str`].
impl Default for Options {
    fn default() -> Self {
        Self {
            with_blanket_implementations: false,
        }
    }
}

/// Takes rustdoc JSON and returns a [`Vec`] of [`PublicItem`]s where each
/// [`PublicItem`] is one public item of the crate, i.e. part of the crate's
/// public API. The [`Vec`] is sorted in a way suitable for display to humans,
/// but the exact order is unspecified.
///
/// There exists a convenient `cargo` wrapper for this function found at
/// <https://github.com/Enselic/cargo-public-items> that builds the rustdoc JSON
/// for you and then invokes this function. If you don't want to use that
/// wrapper, use
/// ```bash
/// RUSTDOCFLAGS='-Z unstable-options --output-format json' cargo +nightly doc --lib --no-deps
/// ```
/// to generate the rustdoc JSON that this function takes as input. For
/// reference, the rustdoc JSON format is documented at
/// <https://rust-lang.github.io/rfcs/2963-rustdoc-json.html>. Open PRs and
/// issues for rustdoc JSON itself can be found at
/// <https://github.com/rust-lang/rust/labels/A-rustdoc-json>.
///
/// # Errors
///
/// E.g. if the JSON is invalid.
pub fn sorted_public_items_from_rustdoc_json_str(
    rustdoc_json_str: &str,
    options: Options,
) -> Result<Vec<PublicItem>> {
    let crate_: rustdoc_types::Crate = serde_json::from_str(rustdoc_json_str)?;

    let mut v: Vec<PublicItem> = implementation::public_items_in_crate(&crate_, options).collect();

    v.sort();

    Ok(v)
}

/// One of the basic uses cases is printing a sorted `Vec` of `PublicItem`s. So
/// we implement `Display` for it.
impl std::fmt::Display for PublicItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}{}", self.prefix, self.path, self.suffix)
    }
}
