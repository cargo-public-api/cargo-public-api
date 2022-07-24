use std::path::PathBuf;

use rustdoc_json::BuildOptions as Options;

/// Helper to get the path to a freshly built rustdoc JSON file for the given
/// test-crate.
pub fn rustdoc_json_path_for_crate(test_crate: &str) -> PathBuf {
    // The test framework is unable to capture output from child processes (see
    // https://users.rust-lang.org/t/cargo-doesnt-capture-stderr-in-tests/67045/4),
    // so build quietly to make running tests much less noisy
    rustdoc_json::build(
        Options::default()
            .toolchain("+nightly")
            .manifest_path(&format!("{}/Cargo.toml", test_crate))
            .quiet(true),
    )
    .unwrap()
}

/// Helper to get a String of freshly built rustdoc JSON for the given
/// test-crate.
#[allow(unused)] // It IS used
pub fn rustdoc_json_str_for_crate(test_crate: &str) -> String {
    std::fs::read_to_string(rustdoc_json_path_for_crate(test_crate)).unwrap()
}
