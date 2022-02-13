use std::{fmt::Display, rc::Rc};

use rustdoc_types::{FnDecl, Impl, Item, ItemEnum, Type};

/// This struct represents one public item of a crate. It wraps a single [Item]
/// but adds additional calculated values to make it easier to work with. Its
/// implementation of [Display] corresponds to one line of the output you see
/// from the `public_items` tool.
///
/// This is currently an implementation detail of this crate, but long term it
/// is expected that it will form part of the public API of this crate, but with
/// a much more condensed and strict API surface of course.
#[derive(Debug, Clone)]
pub struct PublicItem<'a> {
    /// The item we are effectively wrapping.
    pub item: &'a Item,

    /// The parent item. If [Self::item] is e.g. an enum variant, then the
    /// parent is an enum. We follow the chain of parents to be able to know the
    /// correct path to an item in the output.
    parent: Option<Rc<PublicItem<'a>>>,
}

impl<'a> PublicItem<'a> {
    pub fn new(item: &'a Item, parent: Option<Rc<PublicItem<'a>>>) -> Self {
        Self { item, parent }
    }

    fn path(&'a self) -> Vec<Rc<PublicItem<'a>>> {
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

    fn prefix_for_item(&'a self) -> String {
        format!("pub {} ", type_string_for_item(self.item))
    }

    fn suffix_for_item(&'a self) -> String {
        match &self.item.inner {
            ItemEnum::Function(f) => fn_decl_to_string(&f.decl),
            ItemEnum::Method(m) => fn_decl_to_string(&m.decl),
            ItemEnum::Macro(_) | ItemEnum::ProcMacro(_) => String::from("!"),
            _ => String::default(),
        }
    }
}

pub fn type_string_for_item(item: &Item) -> &str {
    match &item.inner {
        ItemEnum::Module(_) => "mod",
        ItemEnum::ExternCrate { .. } => "extern crate",
        ItemEnum::Import(_) => "use",
        ItemEnum::Union(_) => "union",
        ItemEnum::Struct(_) => "struct",
        ItemEnum::StructField(_) => "struct field",
        ItemEnum::Enum(_) => "enum",
        ItemEnum::Variant(_) => "enum variant",
        ItemEnum::Function(_) | ItemEnum::Method(_) => "fn",
        ItemEnum::Trait(_) => "trait",
        ItemEnum::TraitAlias(_) => "trait alias",
        ItemEnum::Impl(_) => "impl",
        ItemEnum::Typedef(_) | ItemEnum::AssocType { .. } => "type",
        ItemEnum::OpaqueTy(_) => "opaque ty",
        ItemEnum::Constant(_) | ItemEnum::AssocConst { .. } => "const",
        ItemEnum::Static(_) => "static",
        ItemEnum::ForeignType => "foreign type",
        ItemEnum::Macro(_) => "macro",
        ItemEnum::ProcMacro(_) => "proc macro",
        ItemEnum::PrimitiveType(name) => name,
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

impl<'a> Display for PublicItem<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let path = self
            .path()
            .iter()
            .map(|i| get_effective_name(i.item))
            .collect::<Vec<_>>();

        write!(
            f,
            "{}{}{}",
            self.prefix_for_item(),
            path.join("::"),
            self.suffix_for_item()
        )
    }
}
