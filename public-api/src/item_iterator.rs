use std::{collections::HashMap, rc::Rc};

use rustdoc_types::{Crate, Id, Impl, Import, Item, ItemEnum, Module, Struct, StructKind, Type};

use super::intermediate_public_item::IntermediatePublicItem;
use crate::{public_item::PublicItem, render::RenderingContext, Options, PublicApi};

type Impls<'a> = HashMap<&'a Id, Vec<&'a Impl>>;

#[derive(Debug, Clone, PartialEq, Eq)]
enum ImplKind {
    Normal,
    AutoTrait,
    Blanket,
}

#[derive(Debug, Clone)]
struct ImplItem<'a> {
    item: &'a Item,
    impl_: &'a Impl,
    for_id: Option<&'a Id>,
    kind: ImplKind,
}

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
    /// The original and unmodified rustdoc JSON, in deserialized form.
    crate_: &'a Crate,

    /// What items left to visit (and possibly add more items from)
    items_left: Vec<Rc<IntermediatePublicItem<'a>>>,

    /// Given a rustdoc JSON Id, keeps track of what public items that have this
    /// ID. The reason this is a one-to-many mapping is because of re-exports.
    /// If an API re-exports a public item in a different place, the same item
    /// will be reachable by different paths, and thus the Vec will contain many
    /// [`IntermediatePublicItem`]s for that ID.
    ///
    /// You might think this is rare, but it is actually more common than you
    /// think out in the wild.
    id_to_items: HashMap<&'a Id, Vec<Rc<IntermediatePublicItem<'a>>>>,

    /// Normally, an item referenced by item Id is present in the rustdoc JSON.
    /// If [`Self::crate_.index`] is missing an Id, then we add it here, to aid
    /// with debugging. It will typically be missing because of bugs (or
    /// borderline bug such as re-exports of foreign items like discussed in
    /// <https://github.com/rust-lang/rust/pull/99287#issuecomment-1186586518>)
    /// We do not report it to users, because they can't do anything about it.
    missing_ids: Vec<&'a Id>,

    /// `impl`s are a bit special. They do not need to be reachable by the crate
    /// root in order to matter. All that matters is that the trait and type
    /// involved are both public.
    ///
    /// Since the rustdoc JSON by definition only includes public items, all
    /// `impl`s we see are potentially relevant. We do some filtering though.
    /// For example, we do not care about blanket implementations by default.
    ///
    /// Whenever we encounter an active `impl` for a type, we inject the
    /// associated items of the `impl` as children of the type.
    active_impls: Impls<'a>,
}

impl<'a> ItemIterator<'a> {
    pub fn new(crate_: &'a Crate, options: Options) -> Self {
        let all_impls: Vec<ImplItem> = all_impls(crate_).collect();

        let mut s = ItemIterator {
            crate_,
            items_left: vec![],
            id_to_items: HashMap::new(),
            missing_ids: vec![],
            active_impls: active_impls(all_impls.clone(), options),
        };

        // Bootstrap with the root item
        s.try_add_item_to_visit(&crate_.root, None);

        // Many `impl`s are not reachable from the root, but we want to list
        // some of them as part of the public API.
        s.try_add_relevant_impls(all_impls);

        s
    }

    fn try_add_relevant_impls(&mut self, all_impls: Vec<ImplItem<'a>>) {
        for impl_ in all_impls {
            // Currently only Auto Trait Implementations are supported/listed
            if impl_.kind == ImplKind::AutoTrait {
                self.try_add_item_to_visit(&impl_.item.id, None);
            }
        }
    }

    fn add_children_for_item(&mut self, public_item: &Rc<IntermediatePublicItem<'a>>) {
        // Handle any impls. See [`ItemIterator::impls`] docs for more info.
        let mut add_after_borrow = vec![];
        if let Some(impls) = self.active_impls.get(&public_item.item.id) {
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
            Some(item) => self.maybe_add_item_to_visit(item, parent),
            None => self.add_missing_id(id),
        }
    }

    fn maybe_add_item_to_visit(
        &mut self,
        item: &'a Item,
        parent: Option<Rc<IntermediatePublicItem<'a>>>,
    ) {
        // We try to inline glob imports, but that might fail, and we want to
        // keep track of when that happens.
        let mut glob_import_inlined = false;

        // We need to handle `pub use foo::*` specially. In case of such
        // wildcard imports, `glob` will be `true` and `id` will be the
        // module we should import all items from, but we should NOT add
        // the module itself.
        if let ItemEnum::Import(Import {
            id: Some(mod_id),
            glob: true,
            ..
        }) = &item.inner
        {
            // Before we inline this wildcard import, make sure that the module
            // is not indirectly trying to import itself. If we allow that,
            // we'll get a stack overflow. Note that `glob_import_inlined`
            // remains `false` in that case, which means that the output will
            // use a special syntax to indicate that we broke recursion.
            if let Some(Item {
                inner: ItemEnum::Module(Module { items, .. }),
                ..
            }) = self.get_item_if_not_in_path(&parent, mod_id)
            {
                for item in items {
                    self.try_add_item_to_visit(item, parent.clone());
                }
                glob_import_inlined = true;
            }
        }

        // if we inlined a glob import earlier, we should not add the import
        // item itself. All other items we can go ahead and add.
        if !glob_import_inlined {
            self.add_item_to_visit(item, parent);
        }
    }

    /// Add an item to the list of items to visit later. Handles imports specially.
    fn add_item_to_visit(
        &mut self,
        original_item: &'a Item,
        parent: Option<Rc<IntermediatePublicItem<'a>>>,
    ) {
        // Normally we add the original item, but in the case of imports we
        // replace this with the *imported* item.
        let mut actual_item = original_item;

        // Imports can optionally rename items.
        let mut overridden_name = None;

        // Since public imports are part of the public API, we inline them, i.e.
        // replace the item corresponding to an import with the item that is
        // imported. If we didn't do this, publicly imported items would show up
        // as just e.g. `pub use some::function`, which is not sufficient for
        // the use cases of this tool. We want to show the actual API, and thus
        // also show type information! There is one exception; for re-exports of
        // primitive types, there is no item Id to inline with, so they remain
        // as e.g. `pub use my_i32` in the output.
        if let ItemEnum::Import(import) = &original_item.inner {
            overridden_name = if import.glob {
                // Items should have been inlined in maybe_add_item_to_visit(),
                // but since we got here that must have failed, typically
                // because the built rustdoc JSON omitted some items from the
                // output, or to break import recursion.
                Some(format!("<<{}::*>>", import.source))
            } else {
                if let Some(imported_item) = import
                    .id
                    .as_ref()
                    .and_then(|imported_id| self.get_item_if_not_in_path(&parent, imported_id))
                {
                    actual_item = imported_item;
                }

                Some(import.name.clone())
            };
        }

        self.just_add_item_to_visit(actual_item, overridden_name, parent);
    }

    /// Adds an item to visit. No questions asked.
    fn just_add_item_to_visit(
        &mut self,
        item: &'a Item,
        overridden_name: Option<String>,
        parent: Option<Rc<IntermediatePublicItem<'a>>>,
    ) {
        let public_item = Rc::new(IntermediatePublicItem {
            item,
            overridden_name,
            parent,
        });

        self.id_to_items
            .entry(&item.id)
            .or_default()
            .push(public_item.clone());

        self.items_left.push(public_item);
    }

    /// Get the rustdoc JSON item with `id`, but only if it is not already part
    /// of the path. This can happen in the case of recursive re-exports, in
    /// which case we need to break the recursion.
    fn get_item_if_not_in_path(
        &mut self,
        parent: &Option<Rc<IntermediatePublicItem<'a>>>,
        id: &'a Id,
    ) -> Option<&'a Item> {
        if parent.clone().map_or(false, |p| p.path_contains_id(id)) {
            // The item is already in the path! Break import recursion...
            return None;
        }

        self.crate_.index.get(id).or_else(|| {
            self.add_missing_id(id);
            None
        })
    }

    fn add_missing_id(&mut self, id: &'a Id) {
        self.missing_ids.push(id);
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

fn all_impls(crate_: &Crate) -> impl Iterator<Item = ImplItem> {
    crate_.index.values().filter_map(|item| match &item.inner {
        ItemEnum::Impl(impl_) => Some(ImplItem {
            item,
            impl_,
            kind: impl_kind(impl_),
            for_id: match &impl_.for_ {
                Type::ResolvedPath(path) => Some(&path.id),
                _ => None,
            },
        }),
        _ => None,
    })
}

const fn impl_kind(impl_: &Impl) -> ImplKind {
    let has_blanket_impl = matches!(impl_.blanket_impl, Some(_));

    // See https://github.com/rust-lang/rust/blob/54f20bbb8a7aeab93da17c0019c1aaa10329245a/src/librustdoc/json/conversions.rs#L589-L590
    match (impl_.synthetic, has_blanket_impl) {
        (true, false) => ImplKind::AutoTrait,
        (false, true) => ImplKind::Blanket,
        _ => ImplKind::Normal,
    }
}

fn active_impls(all_impls: Vec<ImplItem>, options: Options) -> Impls {
    let mut impls = HashMap::new();

    for impl_item in all_impls {
        let for_id = match impl_item.for_id {
            Some(id) => id,
            None => continue,
        };

        let active = match impl_item.kind {
            ImplKind::Blanket => options.with_blanket_implementations,
            ImplKind::AutoTrait | ImplKind::Normal => true,
        };

        if active {
            impls
                .entry(for_id)
                .or_insert_with(Vec::new)
                .push(impl_item.impl_);
        }
    }

    impls
}

/// Some items contain other items, which is relevant for analysis. Keep track
/// of such relationships.
const fn items_in_container(item: &Item) -> Option<&Vec<Id>> {
    match &item.inner {
        ItemEnum::Module(m) => Some(&m.items),
        ItemEnum::Union(u) => Some(&u.fields),
        ItemEnum::Struct(Struct {
            kind: StructKind::Plain { fields, .. },
            ..
        })
        | ItemEnum::Variant(rustdoc_types::Variant::Struct { fields, .. }) => Some(fields),
        ItemEnum::Enum(e) => Some(&e.variants),
        ItemEnum::Trait(t) => Some(&t.items),
        ItemEnum::Impl(i) => Some(&i.items),
        _ => None,
    }
}

pub fn public_api_in_crate(crate_: &Crate, options: Options) -> super::PublicApi {
    let mut item_iterator = ItemIterator::new(crate_, options);
    let items: Vec<_> = item_iterator.by_ref().collect();

    let context = RenderingContext {
        crate_,
        id_to_items: item_iterator.id_to_items,
    };

    PublicApi {
        items: items
            .iter()
            .map(|item| PublicItem::from_intermediate_public_item(&context, item))
            .collect::<Vec<_>>(),
        missing_item_ids: item_iterator
            .missing_ids
            .iter()
            .map(|m| m.0.clone())
            .collect(),
    }
}
