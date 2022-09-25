// deny in CI, only warn here
#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::missing_panics_doc)]
#![allow(dead_code)]

use std::path::PathBuf;

mod create_test_git_repo;
pub use create_test_git_repo::create_test_git_repo;

pub mod assert_or_bless;

/// Helper to get the path to a freshly built rustdoc JSON file for the given
/// test-crate.
#[must_use]
pub fn rustdoc_json_path_for_crate(test_crate: &str) -> PathBuf {
    // The test framework is unable to capture output from child processes (see
    // https://users.rust-lang.org/t/cargo-doesnt-capture-stderr-in-tests/67045/4),
    // so build quietly to make running tests much less noisy

    rustdoc_json::Builder::default()
        .toolchain("+nightly".to_owned())
        .manifest_path(&format!("{}/Cargo.toml", test_crate))
        .quiet(true)
        .build()
        .unwrap()
}

/// Helper to get a String of freshly built rustdoc JSON for the given
/// test-crate.
#[must_use]
#[allow(dead_code)]
pub fn rustdoc_json_str_for_crate(test_crate: &str) -> String {
    std::fs::read_to_string(rustdoc_json_path_for_crate(test_crate)).unwrap()
}
