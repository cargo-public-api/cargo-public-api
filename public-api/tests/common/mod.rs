#![allow(unused)] // Used from many crates, but not all crates use all functions.

use std::path::{Path, PathBuf};

pub fn builder_for_crate(
    test_crate: impl AsRef<Path>,
    target_dir: impl AsRef<Path>,
) -> public_api::Builder {
    let json = rustdoc_json_path_for_crate(test_crate, target_dir);
    public_api::Builder::from_rustdoc_json(json)
}

/// Returns a builder for a so called "simplified" API, which is an API without
/// Auto Trait or Blanket impls, to reduce public item noise.
pub fn simplified_builder_for_crate(
    test_crate: impl AsRef<Path>,
    target_dir: impl AsRef<Path>,
) -> public_api::Builder {
    builder_for_crate(test_crate, target_dir)
        .omit_blanket_impls(true)
        .omit_auto_trait_impls(true)
}

/// Builds rustdoc JSON for the given test crate.
///
/// Output of child processes are not captured by the Rust test framework (see
/// <https://users.rust-lang.org/t/cargo-doesnt-capture-stderr-in-tests/67045/4>),
/// so build quietly to avoid noisy test output.
pub fn rustdoc_json_path_for_crate(
    test_crate: impl AsRef<Path>,
    target_dir: impl AsRef<Path>,
) -> PathBuf {
    let mut manifest_path = test_crate.as_ref().to_path_buf();
    manifest_path.push("Cargo.toml");

    rustdoc_json::Builder::default()
        .manifest_path(&manifest_path)
        .toolchain("nightly")
        .target_dir(target_dir)
        .quiet(true)
        .build()
        .unwrap_or_else(|_| {
            panic!(
                "Failed to build rustdoc JSON for {:?} (current dir: {:?})",
                test_crate.as_ref(),
                std::env::current_dir()
            )
        })
}

/// Builds rustdoc JSON for the given temporary test crate. A temporary test
/// crate is a crate set up in a temporary directory, so that the target dir and
/// root dir can be the same while still allowing tests to run concurrently.
pub fn rustdoc_json_path_for_temp_crate(temp_crate: impl AsRef<Path>) -> PathBuf {
    rustdoc_json_path_for_crate(temp_crate.as_ref(), temp_crate.as_ref())
}
