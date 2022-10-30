use super::intermediate_public_item::NameableItem;
use crate::{
    crate_wrapper::CrateWrapper, intermediate_public_item::IntermediatePublicItem,
    public_item::PublicItem, render::RenderingContext, Options, PublicApi,
};
use rustdoc_types::{Crate, Id, Impl, Import, Item, ItemEnum, Module, Struct, StructKind};
use std::{
    collections::{HashMap, VecDeque},
    vec,
};

/// Items in rustdoc JSON reference each other by Id. The [`ItemProcessor`]
/// essentially takes one Id at a time and figure out what to do with it. Once
/// complete, the item both ready to be listed as part of the public API, and
/// and many cases also ready to be used as part of a path to another item.
///
/// This struct contains a (processed) path to an item that is about to be
/// processed further.
#[derive(Debug)]
struct UnprocessedItem<'c> {
    /// The path to the item to process.
    parent_path: Vec<NameableItem<'c>>,

    /// The Id of the item to process.
    id: &'c Id,
}

/// Processes items to find more items and to figure out the path to each item.
/// Some non-obvious cases to take into consideration are:
/// 1. A single item is imported several times.
/// 2. An item is (publicly) imported from another crate
///
/// Note that this implementation iterates over everything, so if the rustdoc
/// JSON is generated with `--document-private-items`, then private items will
/// also be included in the output. Use with `--document-private-items` is not
/// supported.
pub struct ItemProcessor<'c> {
    /// The original and unmodified rustdoc JSON, in deserialized form.
    crate_: CrateWrapper<'c>,

    /// To know if e.g. blanket implementation should be included in the output.
    options: Options,

    /// A queue of unprocessed items to process.
    work_queue: VecDeque<UnprocessedItem<'c>>,

    /// The output. A list of processed items. Note that the order is
    /// intentionally "logical", so that e.g. struct fields items follows from
    /// struct items.
    output: Vec<IntermediatePublicItem<'c>>,
}

impl<'c> ItemProcessor<'c> {
    pub fn new(crate_: &'c Crate, options: Options) -> Self {
        ItemProcessor {
            crate_: CrateWrapper::new(crate_),
            options,
            work_queue: VecDeque::new(),
            output: vec![],
        }
    }

    fn add_to_work_queue(&mut self, parent_path: Vec<NameableItem<'c>>, id: &'c Id) {
        self.work_queue
            .push_front(UnprocessedItem { parent_path, id });
    }

    /// Processes the entire work queue. Adds more items based on items it
    /// processes. When this returns, all items have been recursively processed.
    fn run(&mut self) {
        while let Some(unprocessed_item) = self.work_queue.pop_front() {
            if let Some(item) = self.crate_.get_item(unprocessed_item.id) {
                self.process_any_item(item, unprocessed_item);
            }
        }
    }

    /// Process any item. In particular, does the right thing is the item is an
    /// impl or an import.
    fn process_any_item(&mut self, item: &'c Item, unprocessed_item: UnprocessedItem<'c>) {
        match &item.inner {
            ItemEnum::Import(import) => {
                if import.glob {
                    self.process_import_glob_item(import, unprocessed_item, item);
                } else {
                    self.process_import_item(item, import, unprocessed_item);
                }
            }
            ItemEnum::Impl(impl_) => {
                self.process_impl_item(unprocessed_item, item, impl_);
            }
            _ => {
                self.process_item(unprocessed_item, item, None);
            }
        }
    }

    /// import all items from, but we should NOT add the module itself. Before
    /// we inline this wildcard import, make sure that the module is not
    /// indirectly trying to import itself. If we allow that, we'll get a stack
    /// overflow.
    fn process_import_glob_item(
        &mut self,
        import: &'c Import,
        unprocessed_item: UnprocessedItem<'c>,
        item: &'c Item,
    ) {
        // Before we inline this wildcard import, make sure that the module is
        // not indirectly trying to import itself. If we allow that, we'll get a
        // stack overflow.
        if let Some(Item {
            inner: ItemEnum::Module(Module { items, .. }),
            ..
        }) = import
            .id
            .as_ref()
            .and_then(|id| self.get_item_if_not_in_path(&unprocessed_item.parent_path, id))
        {
            for item_id in items {
                self.add_to_work_queue(unprocessed_item.parent_path.clone(), item_id);
            }
        } else {
            self.process_item(
                unprocessed_item,
                item,
                Some(format!("<<{}::*>>", import.source)),
            );
        }
    }

    /// Since public imports are part of the public API, we inline them, i.e.
    /// replace the item corresponding to an import with the item that is
    /// imported. If we didn't do this, publicly imported items would show up as
    /// just e.g. `pub use some::function`, which is not sufficient for the use
    /// cases of this tool. We want to show the actual API, and thus also show
    /// type information! There is one exception; for re-exports of primitive
    /// types, there is no item Id to inline with, so they remain as e.g. `pub
    /// use my_i32` in the output.
    fn process_import_item(
        &mut self,
        item: &'c Item,
        import: &'c Import,
        unprocessed_item: UnprocessedItem<'c>,
    ) {
        let mut actual_item = item;

        if let Some(imported_item) = import
            .id
            .as_ref()
            .and_then(|id| self.get_item_if_not_in_path(&unprocessed_item.parent_path, id))
        {
            actual_item = imported_item;
        }

        self.process_item(unprocessed_item, actual_item, Some(import.name.clone()));
    }

    /// Processes impls. Is special only because we support filtering out e.g.
    /// blanket implementations to reduce noise.
    fn process_impl_item(
        &mut self,
        unprocessed_item: UnprocessedItem<'c>,
        item: &'c Item,
        impl_: &'c Impl,
    ) {
        if !ImplKind::from(impl_).is_active(self.options) {
            return;
        }

        self.process_item(unprocessed_item, item, None);
    }

    /// Process an item. Setup jobs for its children and impls and and then put
    /// it in the output.
    fn process_item(
        &mut self,
        unprocessed_item: UnprocessedItem<'c>,
        item: &'c Item,
        overridden_name: Option<String>,
    ) {
        let finished_item = unprocessed_item.finish(item, overridden_name);
        let children = items_in_container(item).into_iter().flatten();
        let impls = impls_for_item(item).into_iter().flatten();

        // Use .rev() so order is preserved with .push_front(). Use
        // .push_front() so that e.g items for struct fields are finished right
        // after their corresponding struct is finished.
        for id in children.chain(impls).rev() {
            self.add_to_work_queue(finished_item.path().into(), id);
        }

        self.output.push(finished_item);
    }

    /// Get the rustdoc JSON item with `id`, but only if it is not already part
    /// of the path. This can happen in the case of recursive re-exports, in
    /// which case we need to break the recursion.
    fn get_item_if_not_in_path(
        &mut self,
        parent_path: &[NameableItem<'c>],
        id: &'c Id,
    ) -> Option<&'c Item> {
        if parent_path.iter().any(|m| m.item.id == *id) {
            // The item is already in the path! Break import recursion...
            return None;
        }

        self.crate_.get_item(id)
    }
}

impl<'c> UnprocessedItem<'c> {
    /// Finishes an item. Turns an [`UnprocessedItem`] into a finished
    /// [`IntermediatePublicItem`].
    fn finish(self, item: &'c Item, overridden_name: Option<String>) -> IntermediatePublicItem<'c> {
        // Transfer path ownership to output item
        let mut path = self.parent_path;

        // Complete the path with the last item
        path.push(NameableItem {
            item,
            overridden_name,
        });

        // Done
        IntermediatePublicItem::new(path)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ImplKind {
    Normal,
    AutoTrait,
    Blanket,
}

impl ImplKind {
    fn from(impl_: &Impl) -> Self {
        let has_blanket_impl = matches!(impl_.blanket_impl, Some(_));

        // See https://github.com/rust-lang/rust/blob/54f20bbb8a7aeab93da17c0019c1aaa10329245a/src/librustdoc/json/conversions.rs#L589-L590
        match (impl_.synthetic, has_blanket_impl) {
            (true, false) => ImplKind::AutoTrait,
            (false, true) => ImplKind::Blanket,
            _ => ImplKind::Normal,
        }
    }

    fn is_active(&self, options: Options) -> bool {
        match self {
            ImplKind::Blanket => options.with_blanket_implementations,
            ImplKind::AutoTrait | ImplKind::Normal => true,
        }
    }
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

pub fn impls_for_item(item: &Item) -> Option<&[Id]> {
    match &item.inner {
        ItemEnum::Union(union_) => Some(&union_.impls),
        ItemEnum::Struct(struct_) => Some(&struct_.impls),
        ItemEnum::Enum(enum_) => Some(&enum_.impls),
        ItemEnum::Primitive(primitive) => Some(&primitive.impls),
        ItemEnum::Trait(trait_) => Some(&trait_.implementations),
        _ => None,
    }
}

pub fn public_api_in_crate(crate_: &Crate, options: Options) -> super::PublicApi {
    let mut item_processor = ItemProcessor::new(crate_, options);
    item_processor.add_to_work_queue(vec![], &crate_.root);
    item_processor.run();

    // Given a rustdoc JSON Id, keeps track of what public items that have this
    // ID. The reason this is a one-to-many mapping is because of re-exports.
    // If an API re-exports a public item in a different place, the same item
    // will be reachable by different paths, and thus the Vec will contain many
    // [`IntermediatePublicItem`]s for that ID.
    //
    // You might think this is rare, but it is actually a common thing in
    // real-world code.
    let mut id_to_items: HashMap<&Id, Vec<&IntermediatePublicItem>> = HashMap::new();
    for finished_item in &item_processor.output {
        id_to_items
            .entry(&finished_item.item().id)
            .or_default()
            .push(finished_item);
    }

    let context = RenderingContext {
        crate_,
        id_to_items,
    };

    PublicApi {
        items: item_processor
            .output
            .iter()
            .map(|item| PublicItem::from_intermediate_public_item(&context, item))
            .collect::<Vec<_>>(),
        missing_item_ids: item_processor.crate_.missing_item_ids(),
    }
}
