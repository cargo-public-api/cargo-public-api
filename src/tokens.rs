pub use rustdoc_types::{GenericBound, Generics, Type, Variant};

#[derive(Clone)]
pub struct PublicItemTokenStream {
    pub qualifiers: Vec<Qualifier>,
    pub path: Vec<String>,
    pub kind: Kind,
}

#[derive(Clone)]
pub enum Qualifier {
    Pub,
    Const,
    Async,
}

impl std::fmt::Display for Qualifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Qualifier::Pub => "pub",
                Qualifier::Const => "const",
                Qualifier::Async => "async",
            }
        )
    }
}

#[derive(Clone)]
pub enum Kind {
    Function {
        generics: Generics,
        arguments: Vec<(String, Type)>,
        return_type: Option<Type>,
    },
    Enum,
    EnumVariant(Variant),
    Struct,
    StructField(Type),
}
