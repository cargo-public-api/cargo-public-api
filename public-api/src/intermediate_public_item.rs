use crate::render;
use std::rc::Rc;

use rustdoc_types::{Crate, Item, ItemEnum};

use crate::tokens::Token;

/// This struct represents one public item of a crate, but in intermediate form.
/// It wraps a single [Item] but adds additional calculated values to make it
/// easier to work with. Later, one [`Self`] will be converted to exactly one
/// [`crate::PublicItem`].
#[derive(Clone)]
pub struct IntermediatePublicItem<'a> {
    /// The item we are effectively wrapping.
    pub item: &'a Item,
    pub root: &'a Crate,

    /// The parent item. If [Self::item] is e.g. an enum variant, then the
    /// parent is an enum. We follow the chain of parents to be able to know the
    /// correct path to an item in the output.
    parent: Option<Rc<IntermediatePublicItem<'a>>>,
}

impl<'a> IntermediatePublicItem<'a> {
    #[must_use]
    pub fn new(
        item: &'a Item,
        root: &'a Crate,
        parent: Option<Rc<IntermediatePublicItem<'a>>>,
    ) -> Self {
        Self { item, root, parent }
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

    /// Some items do not use item.name. Handle that.
    #[must_use]
    pub fn get_effective_name(&'a self) -> String {
        match &self.item.inner {
            // An import uses its own name (which can be different from the name of
            // the imported item)
            ItemEnum::Import(i) => &i.name,

            _ => self.item.name.as_deref().unwrap_or("<<no_name>>"),
        }
        .to_owned()
    }

    pub fn render_token_stream(&self) -> Vec<Token> {
        render::token_stream(self)
    }
}
