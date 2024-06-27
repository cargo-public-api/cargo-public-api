//! This crate is not used in regression tests. Its purposes is to make it easy
//! for `cargo public-api` maintainers to experiment with the rustdoc JSON that
//! unstable Rust can produce.
//!
//! The situation is a bit subtle because `cargo public-api` does not support
//! listing the public API of Rust crates that use unstable Rust features. It
//! would be too much work to maintain regression tests for unstable Rust
//! features. However, in order to list the public API of crates that use stable
//! Rust, a nightly toolchain is required, since that is the only way to build
//! rustdoc JSON for a crate.
//!
//! # Example Usage
//!
//! To investigate what exact version of the nightly toolchain that started to
//! support [inherent associated types][1] for rustdoc JSON, we can run the
//! following commands:
//! ```sh
//! $ cargo run -- --toolchain nightly-2023-05-08 --manifest-path testsuite/test-apis/nightly_api/Cargo.toml -sss
//! pub mod nightly_api
//! pub struct nightly_api::StructWithInherentAssociatedType
//! impl nightly_api::StructWithInherentAssociatedType
//! pub type nightly_api::StructWithInherentAssociatedType::InherentAssociatedType = u8
//! pub fn nightly_api::StructWithInherentAssociatedType::inherent_associated_type_output(&self) -> u8
//!
//! $ cargo run -- --toolchain nightly-2023-05-09 --manifest-path testsuite/test-apis/nightly_api/Cargo.toml -sss
//! Error: Failed to parse rustdoc JSON at "/home/martin/src/cargo-public-api/testsuite/test-apis/nightly_api/target/doc/comprehensive_api.json".
//! [...]
//! Caused by:
//!     invalid type: null, expected struct Path at line 1 column 61136
//! ```
//! and as we can see, it was `nightly-2023-05-09` that changed rustdoc JSON
//! format.
//!
//! [1]: https://github.com/rust-lang/rust/pull/109410

#![feature(inherent_associated_types)]

pub struct StructWithInherentAssociatedType;

impl StructWithInherentAssociatedType {
    pub type InherentAssociatedType = u8;

    pub fn inherent_associated_type_output(&self) -> Self::InherentAssociatedType {
        1
    }
}
