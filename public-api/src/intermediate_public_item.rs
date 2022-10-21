use std::rc::Rc;

use rustdoc_types::{Id, Item};

use crate::{public_item::PublicItemPath, render::RenderingContext, tokens::Token};

/// This struct represents one public item of a crate, but in intermediate form.
/// It wraps a single [Item] but adds additional calculated values to make it
/// easier to work with. Later, one [`Self`] will be converted to exactly one
/// [`crate::PublicItem`].
#[derive(Clone, Debug)]
pub struct IntermediatePublicItem<'a> {
    /// The item we are effectively wrapping.
    pub item: &'a Item,

    /// If `Some`, this overrides [Item::name], which happens in the case of
    /// renamed imports (`pub use other::Item as Foo;`).
    pub overridden_name: Option<String>,

    /// The parent item. If [Self::item] is e.g. an enum variant, then the
    /// parent is an enum. We follow the chain of parents to be able to know the
    /// correct path to an item in the output.
    pub parent: Option<Rc<IntermediatePublicItem<'a>>>,
}

impl<'a> IntermediatePublicItem<'a> {
    pub fn name(&self) -> &str {
        self.overridden_name
            .as_deref()
            .or(self.item.name.as_deref())
            .unwrap_or("<<no_name>>")
    }

    #[must_use]
    pub fn path(&'a self) -> Vec<Rc<IntermediatePublicItem<'a>>> {
        let mut path = vec![];

        let rc_self = Rc::new(self.clone());

        path.insert(0, rc_self.clone());

        let mut current_item = rc_self.clone();
        while let Some(parent) = current_item.parent.clone() {
            path.insert(0, parent.clone());
            current_item = parent.clone();
        }

        path
    }

    #[must_use]
    pub fn path_vec(&'a self) -> PublicItemPath {
        self.path().iter().map(|i| i.name().to_owned()).collect()
    }

    #[must_use]
    pub fn path_contains_id(&self, id: &'a Id) -> bool {
        self.path().iter().any(|m| m.item.id == *id)
    }

    #[must_use]
    pub fn path_contains_renamed_item(&self) -> bool {
        self.path().iter().any(|m| m.overridden_name.is_some())
    }

    pub fn render_token_stream(&self, context: &RenderingContext) -> Vec<Token> {
        context.token_stream(self)
    }
}
