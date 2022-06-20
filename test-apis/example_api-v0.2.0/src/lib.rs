#![no_std] // Reduces rustdoc JSON size by 70%

#[derive(Debug)]
#[non_exhaustive]
pub struct Struct {
    pub v1_field: usize,
    pub v2_field: usize,
}

pub struct StructV2 {
    pub field: usize,
}

pub fn function(v1_param: Struct, v2_param: usize) {}
