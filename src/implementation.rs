use rustdoc_types::Crate;
use std::rc::Rc;

use crate::Options;
use crate::PublicItem;

mod item_iterator;
use item_iterator::ItemIterator;

mod intermediate_public_item;
use intermediate_public_item::IntermediatePublicItem;

pub fn public_items_in_crate(
    crate_: &Crate,
    options: Options,
) -> impl Iterator<Item = crate::PublicItem> + '_ {
    ItemIterator::new(crate_, options).map(|p| intermediate_public_item_to_public_item(&p))
}

fn intermediate_public_item_to_public_item(
    public_item: &Rc<IntermediatePublicItem<'_>>,
) -> PublicItem {
    PublicItem {
        prefix: public_item.prefix(),
        path: public_item
            .path()
            .iter()
            .map(|i| i.get_effective_name())
            .collect::<Vec<String>>()
            .join("::"),
        suffix: public_item.suffix(),
    }
}
