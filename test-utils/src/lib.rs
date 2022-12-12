// deny in CI, only warn here
#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::missing_panics_doc)]
#![allow(dead_code)]

use std::path::Path;
use std::path::PathBuf;

#[must_use]
pub fn rustdoc_json_path_for_crate(test_crate: &str, target_dir: impl AsRef<Path>) -> PathBuf {
    rustdoc_json_path_for_crate_impl(test_crate, Some(target_dir.as_ref()), false)
}

#[must_use]
pub fn rustdoc_json_path_for_crate_with_private_items(
    test_crate: &str,
    target_dir: impl AsRef<Path>,
) -> PathBuf {
    rustdoc_json_path_for_crate_impl(test_crate, Some(target_dir.as_ref()), true)
}

/// Helper to get the path to a freshly built rustdoc JSON file for the given
/// test-crate.
#[must_use]
fn rustdoc_json_path_for_crate_impl(
    test_crate: &str,
    target_dir: Option<&Path>,
    document_private_items: bool,
) -> PathBuf {
    let mut builder = rustdoc_json::Builder::default()
        .toolchain("nightly".to_owned())
        .manifest_path(&format!("{}/Cargo.toml", test_crate))
        // The test framework is unable to capture output from child processes (see
        // https://users.rust-lang.org/t/cargo-doesnt-capture-stderr-in-tests/67045/4),
        // so build quietly to make running tests much less noisy
        .quiet(true)
        .document_private_items(document_private_items);

    if let Some(target_dir) = target_dir {
        builder = builder.target_dir(target_dir);
    }

    builder.build().unwrap()
}
