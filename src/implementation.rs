use rustdoc_types::{Crate, Item, ItemEnum};

use crate::Result;
use builder::PublicItemBuilder;

mod builder;

/// Takes rustdoc JSON and returns a [`Vec`] of [`String`]s where each
/// [`String`] is one public item of the crate, i.e. part of the crate's public
/// API. The [`Vec`] is sorted in a way suitable for display to humans, but the
/// exact order is unspecified.
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
/// <https://rust-lang.github.io/rfcs/2963-rustdoc-json.html>.
///
/// # Errors
///
/// E.g. if the JSON is invalid.
pub fn sorted_public_items_from_rustdoc_json_str(rustdoc_json_str: &str) -> Result<Vec<String>> {
    let crate_: Crate = serde_json::from_str(rustdoc_json_str)?;

    let builder = PublicItemBuilder::new(&crate_);

    let mut result: Vec<String> = crate_
        .index
        .values()
        .filter(|item| item_is_relevant(item))
        .map(|item| builder.build_from_item(item))
        .collect();

    result.sort();

    Ok(result)
}

/// Check if an item is relevant to include in the output.
///
/// * Only the items in the root crate (the "current" crate) are relevant.
///
/// * The items of implementations themselves are excluded. It is sufficient to
///   report items _associated_ with implementations.
fn item_is_relevant(item: &Item) -> bool {
    let is_part_of_root_crate = item.crate_id == 0 /* ROOT_CRATE_ID */;

    let is_impl = matches!(item.inner, ItemEnum::Impl(_));

    is_part_of_root_crate && !is_impl
}
