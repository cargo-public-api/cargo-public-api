// Allow stuff that prevents us from testing unidiomatic but valid public APIs
#![allow(
    unused_variables,
    dead_code,
)]

mod private;
pub use private::StructInPrivateMod;

pub mod attributes;

pub mod constants;

pub mod enums;

pub mod exports;

pub mod functions;

pub mod higher_ranked_trait_bounds;

pub mod impls;

pub mod macros;

pub mod statics;

pub mod structs;
pub use structs::Plain;
pub use structs::Plain as RenamedPlain;

pub mod traits;

pub mod typedefs;

pub mod unions;

pub use i32 as my_i32;
pub use u32;

pub extern crate unicode_ident;

// We currently expect rustdoc JSON to not contain these external items, see
// <https://github.com/rust-lang/rust/issues/99513>
pub use unicode_ident::*;

// This explicitly exported item we expect to see in the rustdoc JSON however
pub use unicode_ident::is_xid_start;
