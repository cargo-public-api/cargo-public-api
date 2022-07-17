use crate::render;
use std::rc::Rc;

use rustdoc_types::Item;

use crate::tokens::Token;

/// This struct represents one public item of a crate, but in intermediate form.
/// It wraps a single [Item] but adds additional calculated values to make it
/// easier to work with. Later, one [`Self`] will be converted to exactly one
/// [`crate::PublicItem`].
#[derive(Clone, Debug)]
pub struct IntermediatePublicItem<'a> {
    /// The item we are effectively wrapping.
    pub item: &'a Item,

    /// The name of the item. Normally this is [Item::name]. But in the case of
    /// renamed imports (`pub use other::item as foo;`) it is the new name.
    pub name: &'a str,

    /// The parent item. If [Self::item] is e.g. an enum variant, then the
    /// parent is an enum. We follow the chain of parents to be able to know the
    /// correct path to an item in the output.
    parent: Option<Rc<IntermediatePublicItem<'a>>>,
}

impl<'a> IntermediatePublicItem<'a> {
    #[must_use]
    pub fn new(
        item: &'a Item,
        name: &'a str,
        parent: Option<Rc<IntermediatePublicItem<'a>>>,
    ) -> Self {
        Self { item, name, parent }
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

    pub fn render_token_stream(&self) -> Vec<Token> {
        render::token_stream(self)
    }
}
