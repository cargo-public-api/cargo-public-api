use std::{collections::HashMap, fmt::Debug, fmt::Display, rc::Rc};

use rustdoc_types::{Crate, Id, Impl, Item, ItemEnum, Type};

use super::intermediate_public_item::IntermediatePublicItem;
use crate::{Options, PublicItem};

type Impls<'a> = HashMap<&'a Id, Vec<&'a Impl>>;

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
    impls: Impls<'a>,
}

impl<'a> ItemIterator<'a> {
    pub fn new(crate_: &'a Crate, options: Options) -> Self {
        let mut s = ItemIterator {
            crate_,
            items_left: vec![],
            missing_ids: vec![],
            impls: find_all_impls(crate_, options),
        };

        // Bootstrap with the root item
        s.try_add_item_to_visit(&crate_.root, None);

        s
    }

    fn add_children_for_item(&mut self, public_item: &Rc<IntermediatePublicItem<'a>>) {
        // Handle any impls. See [`ItemIterator::impls`] docs for more info.
        let mut add_after_borrow = vec![];
        if let Some(impls) = self.impls.get(&public_item.item.id) {
            for impl_ in impls {
                for id in &impl_.items {
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
    }

    fn try_add_item_to_visit(
        &mut self,
        id: &'a Id,
        parent: Option<Rc<IntermediatePublicItem<'a>>>,
    ) {
        match self.crate_.index.get(id) {
            // We handle `impl`s specially, and we don't want to process `impl`
            // items directly. See [`ItemIterator::impls`] docs for more info.
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
            self.add_children_for_item(&public_item.clone());

            result = Some(public_item);
        }

        result
    }
}

/// `impl`s are special. This helper finds all `impl`s. See
/// [`ItemIterator::impls`] docs for more info.
fn find_all_impls(crate_: &Crate, options: Options) -> Impls {
    let mut impls = HashMap::new();

    for item in crate_.index.values() {
        if let ItemEnum::Impl(impl_) = &item.inner {
            if let Impl {
                for_: Type::ResolvedPath { id, .. },
                blanket_impl,
                ..
            } = impl_
            {
                let omit = !options.with_blanket_implementations && matches!(blanket_impl, Some(_));
                if !omit {
                    impls.entry(id).or_insert_with(Vec::new).push(impl_);
                }
            }
        }
    }

    impls
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

pub fn public_items_in_crate(
    crate_: &Crate,
    options: Options,
) -> impl Iterator<Item = crate::PublicItem> + '_ {
    ItemIterator::new(crate_, options).map(|p| intermediate_public_item_to_public_item(&p))
}

fn intermediate_public_item_to_public_item(
    public_item: &Rc<IntermediatePublicItem<'_>>,
) -> PublicItem {
    PublicItem(PublicItemInner {
        prefix: public_item.prefix(),
        path: public_item
            .path()
            .iter()
            .map(|i| i.get_effective_name())
            .collect::<Vec<String>>()
            .join("::"),
        suffix: public_item.suffix(),
    })
}

/// To hide implementation details as much as possible from people who casually
/// skims over the code in our lib.rs
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PublicItemInner {
    /// The "pub struct/fn/..." part of an item.
    pub(crate) prefix: String,

    /// The "your_crate::mod_a::mod_b" part of an item.
    pub(crate) path: String,

    /// The type info part, e.g. "(param_a: Type, param_b: OtherType)" for a
    /// `fn`.
    pub(crate) suffix: String,
}

/// One of the basic uses cases is printing a sorted `Vec` of `PublicItem`s. So
/// we implement `Display` for it.
impl Display for PublicItemInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}{}", self.prefix, self.path, self.suffix)
    }
}

/// We want pretty-printing (`"{:#?}"`) of [`crate::diff::PublicItemsDiff`] to print
/// each public item as `Display`, so implement `Debug` with `Display`.
impl Debug for PublicItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}
