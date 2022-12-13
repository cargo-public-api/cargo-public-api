use std::path::{Path, PathBuf};

/// Builds rustdoc JSON for the given test crate.
///
/// Output of child processes are not captured by the Rust test framework (see
/// <https://users.rust-lang.org/t/cargo-doesnt-capture-stderr-in-tests/67045/4>),
/// so build quietly to avoid noisy test output.
pub fn rustdoc_json_path_for_crate(test_crate: &str, target_dir: impl AsRef<Path>) -> PathBuf {
    rustdoc_json::Builder::default()
        .manifest_path(&format!("{}/Cargo.toml", test_crate))
        .toolchain("nightly".to_owned())
        .target_dir(target_dir)
        .quiet(true)
        .build()
        .unwrap()
}
