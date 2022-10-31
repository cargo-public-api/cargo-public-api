use rustdoc_types::Item;

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

    #[must_use]
    pub fn path_vec(&self) -> PublicItemPath {
        self.path()
            .iter()
            .filter_map(NameableItem::name)
            .map(ToOwned::to_owned)
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
