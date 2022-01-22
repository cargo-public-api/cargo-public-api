use std::collections::HashMap;

use rustdoc_types::{Crate, Id, Item, ItemEnum};

/// Map up what items are contained in what items. We can't limit this to
/// just our crate (the root crate) since some traits (e.g. Clone) are
/// defined outside of the root crate.
pub fn build_container_for_item_map(crate_: &Crate) -> HashMap<&Id, &Item> {
    let mut container_for_item = HashMap::new();

    for container in crate_.index.values() {
        if let Some(items) = items_in_container(container) {
            for item in items {
                container_for_item.insert(item, container);
            }
        }
    }

    container_for_item
}

/// Some items contain other items, which is relevant for analysis. Keep track
/// of such relationships.
fn items_in_container(item: &Item) -> Option<&Vec<Id>> {
    match &item.inner {
        ItemEnum::Module(m) => Some(&m.items),
        ItemEnum::Union(u) => Some(&u.fields),
        ItemEnum::Struct(s) => Some(&s.fields),
        ItemEnum::Enum(e) => Some(&e.variants),
        ItemEnum::Trait(t) => Some(&t.items),
        ItemEnum::Impl(i) => Some(&i.items),
        ItemEnum::Variant(rustdoc_types::Variant::Struct(ids)) => Some(ids),
        // TODO: `ItemEnum::Variant(rustdoc_types::Variant::Tuple(ids)) => Some(ids),` when https://github.com/rust-lang/rust/issues/92945 is fixed
        _ => None,
    }
}
