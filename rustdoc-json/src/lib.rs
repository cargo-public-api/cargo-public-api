//! Utilities for working with rustdoc JSON.
//!
//! # Building
//!
//! Use [`rustdoc_json::Builder`][Builder] to build rustdoc JSON. Like this:
//!
//! ```
//! let json_path = rustdoc_json::Builder::default()
//!     .toolchain("nightly".to_owned())
//!     .manifest_path("Cargo.toml")
//!     .build()
//!     .unwrap();
//!
//! println!("Built and wrote rustdoc JSON to {:?}", &json_path);
//! ```
//!
//! A compilable example can be found
//! [here](https://github.com/Enselic/cargo-public-api/blob/main/rustdoc-json/examples/build-rustdoc-json.rs)

// deny in CI, only warn here
#![warn(clippy::all, clippy::pedantic, missing_docs)]

use std::path::PathBuf;

mod build;
mod manifest_parser;

/// replace `rustdoc_json::build(BuildOptions::default().option1().option2().build())` with `rustdoc_json::Builder::default().option1().option2().build()`
#[deprecated(
    note = "replace `rustdoc_json::build(BuildOptions::default().option1().option2().build())` with `rustdoc_json::Builder::default().option1().option2().build()`"
)]
pub struct BuildOptions;

/// Represents all errors that can occur when using [`Builder::build()`].
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum BuildError {
    /// You tried to generate rustdoc JSON for a virtual manifest. That does not
    /// work. You need to point to the manifest of a real package.
    #[error("Manifest must be for an actual package. `{0:?}` is a virtual manifest")]
    VirtualManifest(PathBuf),

    /// A general error. Refer to the attached error message for more info.
    #[error("Failed to build rustdoc JSON. Stderr: {0}")]
    General(String),

    /// An error originating from `cargo_metadata`.
    #[error(transparent)]
    CargoMetadataError(#[from] cargo_metadata::Error),

    /// Some kind of IO error occurred.
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

/// Builds rustdoc JSON. There are many build options. Refer to the docs to
/// learn about them all. See [top-level docs](crate) for an example on how to use this builder.
#[derive(Debug)]
pub struct Builder {
    toolchain: Option<String>,
    manifest_path: PathBuf,
    target_dir: Option<PathBuf>,
    target: Option<String>,
    quiet: bool,
    no_default_features: bool,
    all_features: bool,
    features: Vec<String>,
    package: Option<String>,
    cap_lints: Option<String>,
}

/// Generate rustdoc JSON for a library crate. Returns the path to the freshly
/// built rustdoc JSON file.
///
/// See [crate] for an example on how to use it.
///
/// # Errors
///
/// E.g. if building the JSON fails or if the manifest path does not exist or is
/// invalid.
#[deprecated(note = "use `rustdoc_json::Builder::build()` instead")]
pub fn build(options: Builder) -> Result<PathBuf, BuildError> {
    options.build()
}
