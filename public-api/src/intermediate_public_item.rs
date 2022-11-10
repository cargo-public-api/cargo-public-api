use rustdoc_types::{Item, ItemEnum};

use crate::{
    item_processor::ImplKind, public_item::PublicItemPath, render::RenderingContext, tokens::Token,
};

/// Wraps an [`Item`] and allows us to override its name.
#[derive(Clone, Debug)]
pub struct NameableItem<'c> {
    /// The item we are effectively wrapping.
    pub item: &'c Item,

    /// If `Some`, this overrides [Item::name], which happens in the case of
    /// renamed imports (`pub use other::Item as Foo;`).
    ///
    /// We can't calculate this on-demand, because we can't know the final name
    /// until we have checked if we need to break import recursion.
    pub overridden_name: Option<String>,

    /// See [`sorting_prefix()`] docs for an explanation why we have this.
    pub sorting_prefix: u8,
}

impl<'c> NameableItem<'c> {
    pub fn name(&self) -> Option<&str> {
        self.overridden_name
            .as_deref()
            .or(self.item.name.as_deref())
    }

    pub fn sortable_name(&self) -> String {
        if let Some(name) = self.name() {
            format!("{:0>3}_{:?}", self.sorting_prefix, name)
        } else {
            self.sorting_prefix.to_string()
        }
    }
}

/// This struct represents one public item of a crate, but in intermediate form.
/// Conceptually it wraps a single [`Item`] even though the path to the item
/// consists of many [`Item`]s. Later, one [`Self`] will be converted to exactly
/// one [`crate::PublicItem`].
#[derive(Clone, Debug)]
pub struct IntermediatePublicItem<'c> {
    path: Vec<NameableItem<'c>>,

    /// Whether or not to indent this item one level (4 spaces) in the rendered output.
    pub(crate) indent: bool,
}

impl<'c> IntermediatePublicItem<'c> {
    pub fn new(path: Vec<NameableItem<'c>>, indent: bool) -> Self {
        Self { path, indent }
    }

    #[must_use]
    pub fn item(&self) -> &'c Item {
        self.path().last().expect("path must not be empty").item
    }

    #[must_use]
    pub fn path(&self) -> &[NameableItem<'c>] {
        &self.path
    }

    /// See [`sorting_prefix()`] docs for an explanation why we have this.
    #[must_use]
    pub fn sortable_path(&self) -> PublicItemPath {
        self.path()
            .iter()
            .map(NameableItem::sortable_name)
            .collect()
    }

    #[must_use]
    pub fn path_contains_renamed_item(&self) -> bool {
        self.path().iter().any(|m| m.overridden_name.is_some())
    }

    pub fn render_token_stream(&self, context: &RenderingContext) -> Vec<Token> {
        context.token_stream(self)
    }
}

/// In order for items in the output to be nicely grouped, we add a prefix to
/// each item in the path to an item. That way, sorting on the name (with this
/// prefix) will group items. But we don't want this prefix to be be visible to
/// users of course, so we do this "behind the scenes".
pub(crate) fn sorting_prefix(item: &Item) -> u8 {
    match &item.inner {
        ItemEnum::ExternCrate { .. } => 1,
        ItemEnum::Import(_) => 2,

        ItemEnum::Primitive(_) => 3,

        ItemEnum::Module(_) => 4,

        ItemEnum::Macro(_) => 5,
        ItemEnum::ProcMacro(_) => 6,

        ItemEnum::Enum(_) => 7,
        ItemEnum::Union(_) => 8,
        ItemEnum::Struct(_) => 9,
        ItemEnum::StructField(_) => 10,
        ItemEnum::Variant(_) => 11,

        ItemEnum::Constant(_) => 12,

        ItemEnum::Static(_) => 13,

        ItemEnum::Trait(_) => 14,

        ItemEnum::Function(_) => 15,

        ItemEnum::Typedef(_) => 16,

        ItemEnum::Impl(impl_) => match ImplKind::from(impl_) {
            ImplKind::Normal => 17,
            ImplKind::AutoTrait => 18,
            ImplKind::Blanket => 19,
        },
        ItemEnum::AssocType { .. } => 20,
        ItemEnum::AssocConst { .. } => 21,

        ItemEnum::Method(_) => 22,

        ItemEnum::ForeignType => 23,

        ItemEnum::OpaqueTy(_) => 24,

        ItemEnum::TraitAlias(_) => 25,
    }
}
