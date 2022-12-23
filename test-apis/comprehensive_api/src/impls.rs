use crate::{
    structs::{Plain, Unit, WithLifetimeAndGenericParam},
    traits::{Simple, TraitReferencingOwnAssociatedType, TraitWithGenerics},
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

impl<'a> Plain {
    pub fn s5(&'a self) {}
}

impl<'b> WithLifetimeAndGenericParam<'b, String> {
    pub fn new(unit_ref: &'b Unit, t: String) -> Self {
        WithLifetimeAndGenericParam { unit_ref, t }
    }
}

impl<'b, T> WithLifetimeAndGenericParam<'b, T> {
    pub fn new_any(unit_ref: &'b Unit, t: T) -> Self {
        WithLifetimeAndGenericParam { unit_ref, t }
    }
}

impl<'b, T> WithLifetimeAndGenericParam<'b, T> {
    pub fn new_any_duplicate(unit_ref: &'b Unit, t: T) -> Self {
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

/// Main purpose is to make it easier to detect regression in how we group items
/// in the (sorted) output
pub struct TestItemGrouping;

impl TraitReferencingOwnAssociatedType for TestItemGrouping {
    type OwnAssociatedType = bool;

    fn own_associated_type_output(&self) -> Self::OwnAssociatedType {
        true
    }

    fn own_associated_type_output_explicit_as(
        &self,
    ) -> <Self as TraitReferencingOwnAssociatedType>::OwnAssociatedType {
        false
    }
}

impl<T, U> TraitWithGenerics<T, U> for TestItemGrouping {
    type Foo = u8;

    fn bar() -> <Self as TraitWithGenerics<T, U>>::Foo {
        1
    }
}
