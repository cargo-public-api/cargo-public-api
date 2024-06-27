use crate::{
    structs::{Plain, Unit, WithLifetimeAndGenericParam},
    traits::{
        GenericAssociatedTypes, Simple, TraitReferencingOwnAssociatedType, TraitWithGenerics,
    },
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

pub struct GatTestStruct1<'a, T>(&'a T);

pub struct GatTestStruct2<T>(T);

impl<'a, T> Simple for GatTestStruct1<'a, T> {
    fn act() {}
}

impl GenericAssociatedTypes for Unit {
    type WhereSelfSized = Self;

    type WhereSimple<T: Simple> = GatTestStruct2<T>;

    type SimpleBound = GatTestStruct1<'static, usize>;

    type WithLifetime<'a> = GatTestStruct1<'a, bool>;
}

/// Regression test for <https://github.com/Enselic/cargo-public-api/issues/429>
pub mod issue_429 {
    pub struct Handle<T>(T);

    pub type HU32 = Handle<u32>;

    impl HU32 {
        pub fn get_u32() -> u32 {
            0
        }
    }
}
