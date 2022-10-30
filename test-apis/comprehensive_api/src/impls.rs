use crate::{
    structs::{Plain, Unit, WithLifetimeAndGenericParam},
    traits::Simple,
};

impl Plain {
    pub fn new() -> Plain {
        Plain { x: 4 }
    }

    pub fn f() {}

    pub fn s1(self) {}

    pub fn s2(&self) {}

    pub fn s3(&mut self) {}
}

impl<'a> Plain {
    pub fn s4(&'a self) {}
}

impl<'b> WithLifetimeAndGenericParam<'b, String> {
    pub fn new(unit_ref: &'b Unit, t: String) -> Self {
        WithLifetimeAndGenericParam { unit_ref, t }
    }
}

impl Simple for Unit {
    fn act() {}
}

pub trait ForUnit {
    fn for_unit();
}

impl ForUnit for () {
    fn for_unit() {}
}
