use std::collections::{HashMap, HashSet};

use rustdoc_types::{Crate, Id, Impl, Item, ItemEnum, Type};

use crate::Result;

mod item_utils;

/// Takes rustdoc JSON and returns a `HashSet` of `String`s where each `String`
/// is a public item of the crate, i.e. part of the crate's public API.
///
/// Use
/// ```bash
/// RUSTDOCFLAGS='-Z unstable-options --output-format json' cargo +nightly doc --lib --no-deps
/// ```
/// to generate rustdoc JSON. The rustdoc JSON format is documented here:
/// <https://rust-lang.github.io/rfcs/2963-rustdoc-json.html>.
///
/// # Errors
///
/// E.g. if the JSON is invalid.
pub fn from_rustdoc_json_str(rustdoc_json_str: &str) -> Result<HashSet<String>> {
    let rustdoc_json: Crate = serde_json::from_str(rustdoc_json_str)?;

    let helper = RustdocJsonHelper::new(&rustdoc_json);

    Ok(helper
        .public_items_in_root_crate()
        .map(|item| helper.full_item_name(item))
        .collect())
}

/// Internal helper to keep track of state while analyzing the JSON
struct RustdocJsonHelper<'a> {
    rustdoc_json: &'a Crate,

    /// Maps an item ID to the container that contains it. Note that the
    /// container itself also is an item. E.g. an enum variant is contained in
    /// an enum item.
    container_for_item: HashMap<&'a Id, &'a Item>,
}

impl<'a> RustdocJsonHelper<'a> {
    fn new(rustdoc_json: &'a Crate) -> RustdocJsonHelper<'a> {
        Self {
            rustdoc_json,
            container_for_item: item_utils::build_container_for_item_map(rustdoc_json),
        }
    }

    fn public_items_in_root_crate(&self) -> impl Iterator<Item = &Item> {
        const ROOT_CRATE_ID: u32 = 0;

        self.rustdoc_json
            .index
            .values()
            .filter(|item| item.crate_id == ROOT_CRATE_ID)
    }

    /// Returns the name of an item, including the path from the crate root.
    fn full_item_name(&self, item: &Item) -> String {
        let mut s = String::new();
        let mut current_item = item;
        loop {
            current_item = if let Some(container) = self.container_for_item(current_item) {
                s = format!("::{}", get_effective_name(current_item)) + &s;
                container
            } else {
                s = get_effective_name(current_item).to_owned() + &s;
                break;
            }
        }
        s
    }

    fn container_for_item(&self, item: &Item) -> Option<&Item> {
        let effective_item_id = get_effective_id(item);
        self.container_for_item.get(effective_item_id).copied()
    }
}

fn get_effective_id(item: &Item) -> &Id {
    match &item.inner {
        ItemEnum::Impl(Impl {
            for_: Type::ResolvedPath { id, .. },
            ..
        }) => id,
        _ => &item.id,
    }
}

/// Some items do not use item.name. Handle that.
fn get_effective_name(item: &Item) -> &str {
    match &item.inner {
        // An import uses its own name (which can be different from the imported name)
        ItemEnum::Import(i) => &i.name,

        // An impl do not have a name. Instead the impl is _for_ something, like
        // a struct. In that case we want the name of the struct (for example).
        ItemEnum::Impl(
            Impl {
                for_: Type::ResolvedPath { name, .. },
                ..
            },
            ..,
        ) => name.as_ref(),

        _ => item.name.as_deref().unwrap_or("<<no_name>>"),
    }
}
