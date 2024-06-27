use std::fmt::{Debug, Display};

pub struct Unit;

pub struct Plain {
    pub x: usize,
}

pub struct PrivateField {
    pub(crate) x: usize,
}

pub struct TupleStructSingle(pub usize);
pub struct TupleStructDouble(pub usize, pub bool);
pub struct TupleStructDoubleWithPrivate(usize, pub bool);
pub struct TupleStructDoubleWithHidden(#[doc(hidden)] usize, pub bool);

pub struct WithLifetimeAndGenericParam<'a, T> {
    pub unit_ref: &'a Unit,
    pub t: T,
}

pub struct ConstArg<T, const N: usize> {
    pub items: [T; N],
}

pub struct OnlyConstArg<const N: usize> {}

pub struct WithTraitBounds<T: Display + Debug> {
    t: T,
}
