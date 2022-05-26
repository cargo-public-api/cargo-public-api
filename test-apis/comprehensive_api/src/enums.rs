use std::fmt::Debug;
use std::fmt::Display;

pub enum SingleVariant {
    Variant,
}

pub enum DiverseVariants {
    Simple,
    Tuple(usize, bool),
    Struct { x: usize, y: SingleVariant },
    Recursive { child: Box<DiverseVariants> },
}

pub enum EnumWithGenerics<'a, T, D: Debug>
where
    T: Display,
{
    Variant { t: &'a T, d: D },
}
