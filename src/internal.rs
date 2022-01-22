use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
};

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
    let crate_: Crate = serde_json::from_str(rustdoc_json_str)?;

    let mut item_displayer = DisplayedItem::new(&crate_);

    Ok(crate_
        .index
        .values()
        .filter(|item| item.crate_id == 0 /* ROOT_CRATE_ID */)
        .map(|item| format!("{}", item_displayer.with(item)))
        .collect())
}

/// Internal helper to keep track of state while analyzing the JSON
struct DisplayedItem<'a> {
    /// Maps an item ID to the container that contains it. Note that the
    /// container itself also is an item. E.g. an enum variant is contained in
    /// an enum item.
    container_for_item: HashMap<&'a Id, &'a Item>,

    /// The current item to display.
    item: Option<&'a Item>,
}

impl<'a> DisplayedItem<'a> {
    fn new(crate_: &'a Crate) -> DisplayedItem<'a> {
        Self {
            container_for_item: item_utils::build_container_for_item_map(crate_),
            item: None,
        }
    }

    fn with(&mut self, item: &'a Item) -> &Self {
        self.item = Some(item);
        self
    }

    fn container_for_item(&self, item: &Item) -> Option<&Item> {
        let effective_item_id = get_effective_id(item);
        self.container_for_item.get(effective_item_id).copied()
    }

    fn path_for_item(&'a self, item: &'a Item) -> Vec<&'a Item> {
        let mut path = vec![];
        path.insert(0, item);

        let mut current_item = item;
        while let Some(container) = self.container_for_item(current_item) {
            path.insert(0, container);
            current_item = container;
        }

        path
    }
}

impl Display for DisplayedItem<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let item = self.item.expect("an item is set");

        let path = self
            .path_for_item(item)
            .iter()
            .map(|i| get_effective_name(i))
            .collect::<Vec<_>>();

        write!(f, "{}", path.join("::"))
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
