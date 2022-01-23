use std::{collections::HashSet, fmt::Display};

use rustdoc_types::{Crate, Item, ItemEnum};

use crate::Result;
use builder::PublicItemBuilder;

mod builder;

/// Represents a public item of a crate. In other words: an item part of the
/// public API of a crate. Note that it implements `Display` and thus can be
/// printed.
#[derive(PartialEq, Eq, Hash)]
pub struct PublicItem {
    /// NOTE: Not visible in the public API of this library
    /// The `"path::to::the::item"`.
    path: String,

    /// NOTE: Not visible in the public API of this library
    /// The part to put before the path, e.g. `pub fn `.
    prefix: String,
}

/// Takes rustdoc JSON and returns a [`HashSet`] of [`PublicItem`]s where each
/// [`PublicItem`] is a public item of the crate, i.e. part of the crate's
/// public API.
///
/// There exists a convenient `cargo` wrapper for this function found at
/// <https://github.com/Enselic/cargo-public-items> that builds the rustdoc JSON
/// for you and then invokes this function. If you don't want to use that
/// wrapper, use
/// ```bash
/// RUSTDOCFLAGS='-Z unstable-options --output-format json' cargo +nightly doc --lib --no-deps
/// ```
/// to generate the rustdoc JSON that this function takes as input. The rustdoc
/// JSON format is documented at
/// <https://rust-lang.github.io/rfcs/2963-rustdoc-json.html>.
///
/// # Errors
///
/// E.g. if the JSON is invalid.
pub fn public_items_from_rustdoc_json_str(rustdoc_json_str: &str) -> Result<HashSet<PublicItem>> {
    let crate_: Crate = serde_json::from_str(rustdoc_json_str)?;

    let builder = PublicItemBuilder::new(&crate_);

    Ok(crate_
        .index
        .values()
        .filter(|item| item_is_relevant(item))
        .map(|item| builder.build_from_item(item))
        .collect())
}

/// Check if an item is relevant to include in the output.
///
/// * Only the items in the root crate (the "current" crate) are relevant.
///
/// * The items of implementations themselves are excluded. It is sufficient to
///   report item _associated_ with implementations.
fn item_is_relevant(item: &Item) -> bool {
    let is_part_of_root_crate = item.crate_id == 0 /* ROOT_CRATE_ID */;
    let is_impl = matches!(item.inner, ItemEnum::Impl(_));
    is_part_of_root_crate && !is_impl
}

impl Display for PublicItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.prefix, self.path)
    }
}
