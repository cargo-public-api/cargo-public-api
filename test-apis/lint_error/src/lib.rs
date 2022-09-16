//! This API should probably be renamed to `problematic_api` since it has more
//! problems than lint problems, but I am too lazy to do that now.
//!
//! Deliberately missing docs to trigger `#![deny(missing_docs)]`. We still want
//! to be able to build rustdoc JSON in this case. In reality we do this for
//! `#![deny(warnings)]` when newer compiler versions comes up with new
//! warnings. But for testing purposes it is fine to use
//! `#![deny(missing_docs)]`.
#![deny(missing_docs)]
#![no_std] // Reduces rustdoc JSON size by 70%

pub struct MissingDocs;

/// Test for verbose output about missing items
pub use unicode_ident;
