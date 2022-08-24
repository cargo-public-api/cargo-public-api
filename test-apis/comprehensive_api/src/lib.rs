pub extern crate rand;
// We expect rustdoc JSON to not contain these external items
pub use rand::distributions::uniform::*;
pub use rand::RngCore;

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
