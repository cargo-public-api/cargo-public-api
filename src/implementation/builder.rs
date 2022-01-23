use std::collections::HashMap;

use rustdoc_types::{Crate, FnDecl, Id, Impl, Item, ItemEnum, Type};

mod item_utils;

/// Internal helper to keep track of state while analyzing the JSON
#[allow(clippy::module_name_repetitions)]
pub struct PublicItemBuilder<'a> {
    /// Maps an item ID to the container that contains it. Note that the
    /// container itself also is an item. E.g. an enum variant item is contained
    /// in an enum item.
    container_for_item: HashMap<&'a Id, &'a Item>,
}

impl<'a> PublicItemBuilder<'a> {
    pub fn new(crate_: &'a Crate) -> PublicItemBuilder<'a> {
        Self {
            container_for_item: item_utils::build_container_for_item_map(crate_),
        }
    }

    pub fn build_from_item(&self, item: &Item) -> String {
        let path = self
            .path_for_item(item)
            .iter()
            .map(|i| get_effective_name(i))
            .collect::<Vec<_>>();

        // Inform users about buggy enum variant tuple struct fields if applicable
        let mut suffix = Self::suffix_for_item(item);
        if path.len() == 1 && matches!(item.inner, ItemEnum::StructField(_)) {
            suffix += " (path missing due to https://github.com/rust-lang/rust/issues/92945)";
        }

        format!(
            "{}{}{}",
            Self::prefix_for_item(item),
            path.join("::"),
            suffix,
        )
    }

    fn container_for_item(&self, item: &Item) -> Option<&Item> {
        let effective_item_id = get_effective_id(item);
        self.container_for_item.get(effective_item_id).copied()
    }

    fn prefix_for_item(item: &Item) -> String {
        format!("pub {} ", item_utils::type_string_for_item(item))
    }

    fn suffix_for_item(item: &Item) -> String {
        match &item.inner {
            ItemEnum::Function(f) => Self::fn_decl_to_string(&f.decl),
            ItemEnum::Method(m) => Self::fn_decl_to_string(&m.decl),
            ItemEnum::Macro(_) | ItemEnum::ProcMacro(_) => String::from("!"),
            _ => String::default(),
        }
    }

    fn fn_decl_to_string(fn_decl: &FnDecl) -> String {
        format!(
            "({})",
            fn_decl
                .inputs
                .iter()
                .map(|i| i.0.clone())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }

    fn path_for_item(&'a self, item: &'a Item) -> Vec<&'a Item> {
        let mut path = vec![];
        path.insert(0, item);

        let mut current_item = item;
        while let Some(container) = self.container_for_item(current_item) {
            path.insert(0, container);
            current_item = container;
        }

        path
    }
}

fn get_effective_id(item: &Item) -> &Id {
    match &item.inner {
        ItemEnum::Impl(Impl {
            for_: Type::ResolvedPath { id, .. },
            ..
        }) => id,
        _ => &item.id,
    }
}

/// Some items do not use item.name. Handle that.
fn get_effective_name(item: &Item) -> &str {
    match &item.inner {
        // An import uses its own name (which can be different from the imported name)
        ItemEnum::Import(i) => &i.name,

        // An impl do not have a name. Instead the impl is _for_ something, like
        // a struct. In that case we want the name of the struct (for example).
        ItemEnum::Impl(
            Impl {
                for_: Type::ResolvedPath { name, .. },
                ..
            },
            ..,
        ) => name.as_ref(),

        _ => item.name.as_deref().unwrap_or("<<no_name>>"),
    }
}
