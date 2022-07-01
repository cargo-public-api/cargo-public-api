//! Utilities for working with rustdoc JSON.
//!
//! Currently only [`build()`] and [`build_quietly()`]. Please see their docs
//! for more info.

#![deny(missing_docs)]

mod build;

use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

/// Represent all errors that can occur when using [`build()`] and
/// [`build_quietly()`].
#[derive(thiserror::Error, Debug)]
pub enum BuildError {
    /// You tried to generate rustdoc JSON for a virtual manifest. That does not
    /// work. You need to point to the manifest of a real package.
    #[error("Manifest must be for an actual package. `{0:?}` is a virtual manifest")]
    VirtualManifest(PathBuf),

    /// A general error. Refer to the attached error message for more info.
    #[error("Failed to build rustdoc JSON. Stderr: {0}")]
    General(String),

    /// An error originating from `cargo_toml`.
    #[error(transparent)]
    CargoTomlError(#[from] cargo_toml::Error),

    /// An error originating from `cargo_metadata`.
    #[error(transparent)]
    CargoMetadataError(#[from] cargo_metadata::Error),

    /// Some kind of IO error occurred.
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

/// Generate rustdoc JSON for a library crate. Returns the path to the freshly
/// built rustdoc JSON file. `toolchain` is the toolchain, e.g. `"+nightly"`.
/// `manifest_path` is the relative or absolute path to `Cargo.toml`.
///
/// # Errors
///
/// E.g. if building the JSON fails or of the manifest path does not exist or is
/// invalid.
pub fn build(
    toolchain: impl AsRef<OsStr>,
    manifest_path: impl AsRef<Path>,
) -> Result<PathBuf, BuildError> {
    build::run_cargo_rustdoc(toolchain, manifest_path, false)
}

/// Same as [`build()`] but also passes `--quiet` to `cargo`.
#[allow(clippy::missing_errors_doc)]
pub fn build_quietly(
    toolchain: impl AsRef<OsStr>,
    manifest_path: impl AsRef<Path>,
) -> Result<PathBuf, BuildError> {
    build::run_cargo_rustdoc(toolchain, manifest_path, true)
}
