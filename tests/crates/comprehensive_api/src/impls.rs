use crate::structs::{Plain, Unit, WithLifetimeAndGenericParam};

impl Plain {
    pub fn new() -> Plain {
        Plain { x: 4 }
    }

    pub fn f() {}

    pub fn s1(self) {}

    pub fn s2(&self) {}

    pub fn s3(&mut self) {}
}

impl<'b> WithLifetimeAndGenericParam<'b, String> {
    pub fn new(unit_ref: &'b Unit, t: String) -> Self {
        WithLifetimeAndGenericParam { unit_ref, t }
    }
}
