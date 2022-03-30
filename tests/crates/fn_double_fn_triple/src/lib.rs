#![no_std] // Reduces rustdoc JSON size by 70%

pub fn double(x: u64) -> u64 {
    x * x
}

pub fn triple(x: u64) -> u64 {
    x * x * x
}
