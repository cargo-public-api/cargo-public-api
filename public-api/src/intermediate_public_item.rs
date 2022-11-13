use rustdoc_types::{Impl, Item, ItemEnum};

use crate::{public_item::PublicItemPath, render::RenderingContext, tokens::Token};

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

    /// See [`crate::item_processor::sorting_prefix()`] docs for an explanation why we have this.
    pub sorting_prefix: u8,
}

impl<'c> NameableItem<'c> {
    /// The regular name of the item. Shown to users.
    pub fn name(&self) -> Option<&str> {
        self.overridden_name
            .as_deref()
            .or(self.item.name.as_deref())
    }

    /// The name that, when sorted on, will group items nicely. Is never shown
    /// to a user.
    pub fn sortable_name(&self) -> String {
        let mut perceived_name = "";

        if let ItemEnum::Impl(Impl {
            trait_: Some(trait_path),
            ..
        }) = &self.item.inner
        {
            // In order for items of impls to be grouped together with its impl, add
            // the "name" of the impl to the sorting prefix.
            perceived_name = &trait_path.name;
        }

        // Note that in order for the prefix to sort properly lexicographically,
        // we need to pad it with leading zeroes.
        let mut sortable_name = format!("{:0>3}{}", self.sorting_prefix, perceived_name);
        if let Some(name) = self.name() {
            sortable_name.push('-');
            sortable_name.push_str(name);
        }
        sortable_name
    }
}

/// This struct represents one public item of a crate, but in intermediate form.
/// Conceptually it wraps a single [`Item`] even though the path to the item
/// consists of many [`Item`]s. Later, one [`Self`] will be converted to exactly
/// one [`crate::PublicItem`].
#[derive(Clone, Debug)]
pub struct IntermediatePublicItem<'c> {
    path: Vec<NameableItem<'c>>,
}

impl<'c> IntermediatePublicItem<'c> {
    pub fn new(path: Vec<NameableItem<'c>>) -> Self {
        Self { path }
    }

    #[must_use]
    pub fn item(&self) -> &'c Item {
        self.path().last().expect("path must not be empty").item
    }

    #[must_use]
    pub fn path(&self) -> &[NameableItem<'c>] {
        &self.path
    }

    /// See [`crate::item_processor::sorting_prefix()`] docs for an explanation why we have this.
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
