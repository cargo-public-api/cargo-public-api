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
}

impl<'c> NameableItem<'c> {
    pub fn name(&self) -> Option<&str> {
        self.overridden_name
            .as_deref()
            .or(self.item.name.as_deref())
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

    #[must_use]
    pub fn path_vec(&self) -> PublicItemPath {
        self.path()
            .iter()
            .filter_map(NameableItem::name)
            .map(ToOwned::to_owned)
            .collect()
    }

    /// See [`sorting_prefix()`] docs for an explanation why we have this.
    #[must_use]
    pub fn sortable_path(&self) -> PublicItemPath {
        self.path()
            .iter()
            .map(|item| format!("{}_{:?}", sorting_prefix(item.item), item.name()))
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
pub(crate) fn sorting_prefix(item: &Item) -> &'static str {
    match &item.inner {
        ItemEnum::ExternCrate { .. } => "01_extern_crate",
        ItemEnum::Import(_) => "02_import",

        ItemEnum::Primitive(_) => "03_prim",

        ItemEnum::Module(_) => "04_mod",

        ItemEnum::Macro(_) => "05_macro",
        ItemEnum::ProcMacro(_) => "06_proc_macro",

        ItemEnum::Enum(_) => "07_enum",
        ItemEnum::Union(_) => "08_union",
        ItemEnum::Struct(_) => "09_struct",
        ItemEnum::StructField(_) => "10_field",
        ItemEnum::Variant(_) => "11_variant",

        ItemEnum::Constant(_) => "12_const",

        ItemEnum::Static(_) => "13_static",

        ItemEnum::Trait(_) => "14_trait",

        ItemEnum::Function(_) => "15_fn",

        ItemEnum::Typedef(_) => "16_typedef",

        ItemEnum::Impl(impl_) => match ImplKind::from(impl_) {
            ImplKind::Normal => "17_impl_01_normal",
            ImplKind::AutoTrait => "17_impl_02_auto_trait",
            ImplKind::Blanket => "17_impl_03_blanket",
        },
        ItemEnum::AssocType { .. } => "18_assoc_type",
        ItemEnum::AssocConst { .. } => "19_assoc_const",

        ItemEnum::Method(_) => "20_method",

        ItemEnum::ForeignType => "21_foreign_type",

        ItemEnum::OpaqueTy(_) => "22_opaque_type",

        ItemEnum::TraitAlias(_) => "23_trait_alias",
    }
}
