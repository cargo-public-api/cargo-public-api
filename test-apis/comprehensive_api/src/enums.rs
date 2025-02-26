use std::fmt::Debug;
use std::fmt::Display;

pub enum SingleVariant {
    Variant,
}

#[repr(u8)]
pub enum SingleVariantReprC {
    Variant,
}

pub enum EnumWithExplicitDiscriminants {
    First = 1,
    Second = 2,
    TenPlusTen = 10 + 10,
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

pub enum EnumWithStrippedTupleVariants {
    Single(usize),
    SingleHidden(#[doc(hidden)] usize),
    Double(bool, bool),
    DoubleFirstHidden(#[doc(hidden)] bool, bool),
    DoubleSecondHidden(bool, #[doc(hidden)] bool),
}
