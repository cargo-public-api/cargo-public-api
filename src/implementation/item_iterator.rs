use std::{collections::HashMap, rc::Rc};

use rustdoc_types::{Crate, Id, Impl, Item, ItemEnum, Type};

mod intermediate_public_item;
pub use intermediate_public_item::IntermediatePublicItem;

/// Iterates over all items in a crate. Iterating over items has the benefit of
/// behaving properly when:
/// 1. A single item is imported several times.
/// 2. An item is (publicly) imported from another crate
///
/// Note that this implementation iterates over everything (with the exception
/// of `impl`s, see relevant code for more details), so if the rustdoc JSON is
/// generated with `--document-private-items`, then private items will also be
/// included in the output.
pub struct ItemIterator<'a> {
    /// The entire rustdoc JSON data
    crate_: &'a Crate,

    /// What items left to visit (and possibly add more items from)
    items_left: Vec<Rc<IntermediatePublicItem<'a>>>,

    /// Normally, a reference in the rustdoc JSON exists. If
    /// [Self::crate_.index] is missing an id (e.g. if it is for a dependency
    /// but the rustdoc JSON was built with `--no-deps`) then we track that in
    /// this field.
    missing_ids: Vec<&'a Id>,

    /// `impl`s are a bit special. They do not need to be reachable by the crate
    /// root in order to matter. All that matters is that the trait and type
    /// involved are both public. Since the rustdoc JSON by definition only
    /// includes public items, all `impl`s we see are relevant. Whenever we
    /// encounter a type that has an `impl`, we inject the associated items of
    /// the `impl` as children of the type.
    impls: HashMap<&'a Id, Vec<&'a Vec<Id>>>,
}

impl<'a> ItemIterator<'a> {
    pub fn new(crate_: &'a Crate) -> Self {
        let mut impls: HashMap<&Id, Vec<&Vec<Id>>> = HashMap::new();
        for item in crate_.index.values() {
            if let ItemEnum::Impl(Impl {
                for_: Type::ResolvedPath { ref id, .. },
                items,
                ..
            }) = &item.inner
            {
                impls.entry(id).or_insert_with(Vec::new).push(items);
            }
        }

        let mut s = ItemIterator {
            crate_,
            items_left: vec![],
            missing_ids: vec![],
            impls,
        };

        s.try_add_item_to_visit(&crate_.root, None);

        s
    }

    fn try_add_item_to_visit(
        &mut self,
        id: &'a Id,
        parent: Option<Rc<IntermediatePublicItem<'a>>>,
    ) {
        match self.crate_.index.get(id) {
            // We handle `impl`s specially, so we don't want to process `impl`
            // items directly. See other comments for more info
            Some(Item {
                inner: ItemEnum::Impl { .. },
                ..
            }) => (),

            Some(item) => self
                .items_left
                .push(Rc::new(IntermediatePublicItem::new(item, parent))),

            None => self.missing_ids.push(id),
        }
    }
}

impl<'a> Iterator for ItemIterator<'a> {
    type Item = Rc<IntermediatePublicItem<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut result = None;

        if let Some(public_item) = self.items_left.pop() {
            // Handle any impls. See field doc for more info
            let mut add_after_borrow = vec![];
            if let Some(impls) = self.impls.get(&public_item.item.id) {
                for items in impls {
                    for id in *items {
                        add_after_borrow.push(id);
                    }
                }
            }
            for id in add_after_borrow {
                self.try_add_item_to_visit(id, Some(public_item.clone()));
            }

            // Handle regular children of the item
            for child in items_in_container(public_item.item).into_iter().flatten() {
                self.try_add_item_to_visit(child, Some(public_item.clone()));
            }

            result = Some(public_item);
        }

        result
    }
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
