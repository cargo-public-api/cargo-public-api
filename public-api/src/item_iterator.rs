use std::{collections::HashMap, fmt::Display, rc::Rc};

use rustdoc_types::{Crate, Id, Impl, Import, Item, ItemEnum, Module, Struct, StructKind, Type};

use super::intermediate_public_item::IntermediatePublicItem;
use crate::{render::RenderingContext, tokens::Token, Options, PublicApi};

type Impls<'a> = HashMap<&'a Id, Vec<&'a Impl>>;

#[derive(Debug, Clone)]
enum ImplKind {
    Normal,
    Blanket,
}

#[derive(Debug, Clone)]
struct ImplItem<'a> {
    impl_: &'a Impl,
    for_id: Option<&'a Id>,
    kind: ImplKind,
}

/// Each public item has a path that is displayed like `first::second::third`.
/// Internally we represent that with a `vec!["first", "second", "third"]`. This
/// is a type alias for that internal representation to make the code easier to
/// read.
pub(crate) type PublicItemPath = Vec<String>;

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
            missing_ids: vec![],
            active_impls: active_impls(all_impls.clone(), options),
        };

        // Bootstrap with the root item
        s.try_add_item_to_visit(&crate_.root, None);

        s
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
            if !parent.clone().map_or(false, |p| p.path_contains_id(mod_id)) {
                if let Some(Item {
                    inner: ItemEnum::Module(Module { items, .. }),
                    ..
                }) = self.crate_.index.get(mod_id)
                {
                    for item in items {
                        self.try_add_item_to_visit(item, parent.clone());
                    }
                    glob_import_inlined = true;
                }
            }
        }

        // We handle `impl`s specially, and we don't want to process `impl`
        // items directly. See [`ItemIterator::impls`] docs for more info. And
        // if we inlined a glob import earlier, we should not add the import
        // item itself. All other items we can go ahead and add.
        if !glob_import_inlined && !matches!(item.inner, ItemEnum::Impl { .. }) {
            self.add_item_to_visit(item, parent);
        }
    }

    fn add_item_to_visit(
        &mut self,
        mut item: &'a Item,
        parent: Option<Rc<IntermediatePublicItem<'a>>>,
    ) {
        let mut name = item.name.clone();

        // Since public imports are part of the public API, we inline them, i.e.
        // replace the item corresponding to an import with the item that is
        // imported. If we didn't do this, publicly imported items would show up
        // as just e.g. `pub use some::function`, which is not sufficient for
        // the use cases of this tool. We want to show the actual API, and thus
        // also show type information! There is one exception; for re-exports of
        // primitive types, there is no item Id to inline with, so they remain
        // as e.g. `pub use my_i32` in the output.
        if let ItemEnum::Import(import) = &item.inner {
            name = if import.glob {
                // Items should have been inlined in maybe_add_item_to_visit(),
                // but since we got here that must have failed, typically
                // because the built rustdoc JSON omitted some items from the
                // output, or to break import recursion.
                Some(format!("<<{}::*>>", import.source))
            } else {
                if let Some(imported_id) = &import.id {
                    if !parent
                        .clone()
                        .map_or(false, |p| p.path_contains_id(imported_id))
                    {
                        match self.crate_.index.get(imported_id) {
                            Some(imported_item) => item = imported_item,
                            None => self.add_missing_id(imported_id),
                        }
                    }
                }
                Some(import.name.clone())
            };
        }

        let public_item = Rc::new(IntermediatePublicItem::new(
            item,
            name.unwrap_or_else(|| String::from("<<no_name>>")),
            parent,
        ));

        self.items_left.push(public_item);
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

fn impl_kind(impl_: &Impl) -> ImplKind {
    let has_blanket_impl = matches!(impl_.blanket_impl, Some(_));

    // See https://github.com/rust-lang/rust/blob/54f20bbb8a7aeab93da17c0019c1aaa10329245a/src/librustdoc/json/conversions.rs#L589-L590
    match (impl_.synthetic, has_blanket_impl) {
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
            ImplKind::Normal => true,
            ImplKind::Blanket => options.with_blanket_implementations,
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
fn items_in_container(item: &Item) -> Option<&Vec<Id>> {
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
    let context = RenderingContext { crate_ };
    let mut item_iterator = ItemIterator::new(crate_, options);
    let items = item_iterator
        .by_ref()
        .map(|p| intermediate_public_item_to_public_item(&context, &p))
        .collect();

    PublicApi {
        items,
        missing_item_ids: item_iterator
            .missing_ids
            .iter()
            .map(|m| m.0.clone())
            .collect(),
    }
}

fn intermediate_public_item_to_public_item(
    context: &RenderingContext,
    public_item: &Rc<IntermediatePublicItem<'_>>,
) -> PublicItem {
    PublicItem {
        path: public_item
            .path()
            .iter()
            .map(|i| i.name.clone())
            .collect::<PublicItemPath>(),
        tokens: public_item.render_token_stream(context),
    }
}

/// Represent a public item of an analyzed crate, i.e. an item that forms part
/// of the public API of a crate. Implements [`Display`] so it can be printed. It
/// also implements [`Ord`], but how items are ordered are not stable yet, and
/// will change in later versions.
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct PublicItem {
    /// The "your_crate::mod_a::mod_b" part of an item. Split by "::"
    pub(crate) path: PublicItemPath,

    /// The rendered item as a stream of [`Token`]s
    pub(crate) tokens: Vec<Token>,
}

impl PublicItem {
    /// The rendered item as a stream of [`Token`]s
    pub fn tokens(&self) -> impl Iterator<Item = &Token> {
        self.tokens.iter()
    }
}

/// We want pretty-printing (`"{:#?}"`) of [`crate::diff::PublicItemsDiff`] to print
/// each public item as `Display`, so implement `Debug` with `Display`.
impl std::fmt::Debug for PublicItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}

/// One of the basic uses cases is printing a sorted `Vec` of `PublicItem`s. So
/// we implement `Display` for it.
impl Display for PublicItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", tokens_to_string(&self.tokens))
    }
}

pub(crate) fn tokens_to_string(tokens: &[Token]) -> String {
    tokens.iter().map(Token::text).collect()
}

impl PartialOrd for PublicItem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PublicItem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.to_string().cmp(&other.to_string())
    }
}
