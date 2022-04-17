use std::fmt::{Debug, Display};

pub struct Unit;

pub struct Plain {
    pub x: usize,
}

pub struct PrivateField {
    pub(crate) x: usize,
}

pub struct Tuple(pub usize);

pub struct WithLifetimeAndGenericParam<'a, T> {
    pub unit_ref: &'a Unit,
    pub t: T,
}

pub struct ConstArg<T, const N: usize> {
    pub items: [T; N],
}

pub struct WithTraitBounds<T: Display + Debug> {
    t: T,
}
