//! The values should not be part of the public API, only the symbols should be.

pub const CONST_STR: &str = "a-str-value-that-itself-is-not-part-of-the-public-api-surface";

pub const CONST_USIZE: usize = 42;

pub const CONST_BOOL: bool = true;

pub const CONST_F64: f64 = 3.1415926535;

pub const CONST_I32_ARRAY: [i32; 3] = [1, 2, 3];

pub const CONST_I32_F64_TUPLE: (i32, f64) = (42, 3.14);

pub const CONST_OPTION_I32: Option<i32> = Some(10);

pub const CONST_PLAIN_STRUCT: crate::structs::Plain = crate::structs::Plain { x: 42 };

pub const CONST_FN: fn(usize) = crate::functions::one_arg;
