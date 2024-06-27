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
                "Failed to build rustdoc JSON for {:?} (current dir: {:?}",
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

pub fn repo_path(relative_path_from_repo_root: &str) -> PathBuf {
    let mut repo_root = std::env::current_dir().unwrap();

    // `find -name testsuite -type d` only gets one hit and it is in the root.
    while !repo_root.join("testsuite").is_dir() {
        let _ = repo_root.pop();
    }
    repo_root.join(relative_path_from_repo_root)
}

pub fn build_public_api(package_name: &str) -> public_api::PublicApi {
    // Install a compatible nightly toolchain if it is missing
    rustup_toolchain::install(public_api::MINIMUM_NIGHTLY_RUST_VERSION).unwrap();

    // Build rustdoc JSON with a separate target dir for increased parallelism
    // and reduced risk of cargo removing files we want to keep
    let target_dir = repo_path("target2/public_api").join(package_name);
    let manifest = repo_path(package_name).join("Cargo.toml");
    let rustdoc_json_path = rustdoc_json::Builder::default()
        .toolchain(public_api::MINIMUM_NIGHTLY_RUST_VERSION)
        .target_dir(&target_dir)
        .manifest_path(&manifest)
        .build()
        .unwrap_or_else(|e| panic!("{e} manifest={manifest:?} target_dir={target_dir:?}"));

    // Derive the public API from the rustdoc JSON
    public_api::Builder::from_rustdoc_json(&rustdoc_json_path)
        .build()
        .unwrap_or_else(|e| {
            panic!(
                "{e} manifest={manifest:?} target_dir={target_dir:?} rustdoc_json={rustdoc_json_path:?}"
            )
        })
}
