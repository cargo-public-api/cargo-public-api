use std::fmt::{Debug, Display};

use crate::{
    structs::{PrivateField, Tuple},
    traits::Simple,
    unions::Basic,
    RenamedPlain,
};

pub fn plain() {}

pub const fn const_fn() {}

pub fn one_arg(x: usize) {
    println!("{}", x);
}

pub fn struct_arg(s: PrivateField) {
    println!("{}", s.x);
}

pub fn fn_arg(f: impl Fn(bool, RenamedPlain) -> bool, mut f_mut: impl FnMut() -> ()) {
    if f(false, RenamedPlain { x: 9 }) {
        f_mut();
    }
}

pub fn return_tuple() -> (bool, Basic) {
    (true, Basic { x: 42 })
}

pub fn return_slice<'a>(input: &'a [usize]) -> &'a [usize] {
    &input
}

pub fn return_raw_pointer(input: &usize) -> *const usize {
    input
}

pub fn return_mut_raw_pointer(input: &mut usize) -> *mut usize {
    input
}

pub fn return_array() -> [u8; 2] {
    [99, 98]
}

pub fn return_iterator() -> impl Iterator<Item = u32> {
    vec![1, 2, 3].into_iter()
}

pub fn generic_arg<T>(t: T) -> T {
    t
}

pub fn generic_bound<T: Sized>(t: T) -> T {
    t
}

pub fn inferred_lifetime(foo: &'_ usize) -> usize {
    *foo
}

pub fn outlives<'a, 'b: 'a, 'c: 'b + 'a>(x: &'a bool, y: &'b i128, z: &'c Tuple) -> usize {
    if *x && *y > 0 {
        z.0
    } else {
        1234
    }
}

pub fn synthetic_arg(t: impl Simple) -> impl Simple {
    t
}

pub fn impl_multiple<T>(t: impl Simple + AsRef<T>) -> impl Simple {}

pub fn somewhere<T, U>(t: T, u: U)
where
    T: Display,
    U: Debug,
{
    println!("{}, {:?}", t, u);
}

pub fn multiple_bounds<T>(t: T)
where
    T: Debug + Display,
{
}

pub fn multiple_bounds_inline<T: Debug + Display>(t: T) {}

pub fn dyn_arg_one_trait(d: &dyn std::io::Write) {}

pub fn dyn_arg_one_trait_one_lifetime(d: &(dyn std::io::Write + 'static)) {}

pub fn dyn_arg_two_traits(d: &(dyn std::io::Write + Send)) {}

pub fn dyn_arg_two_traits_one_lifetime(d: &(dyn std::io::Write + Send + 'static)) {}

pub unsafe fn unsafe_fn() {}

pub async fn async_fn() {}
