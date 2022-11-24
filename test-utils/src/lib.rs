// deny in CI, only warn here
#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::missing_panics_doc)]
#![allow(dead_code)]

use std::path::Path;
use std::path::PathBuf;

mod create_test_git_repo;
use assert_cmd::prelude::OutputOkExt;
pub use create_test_git_repo::create_test_git_repo;

pub mod assert_or_bless;
pub use assert_or_bless::assert_eq_or_bless;
pub use assert_or_bless::write_to_file_atomically;

#[must_use]
pub fn rustdoc_json_path_for_crate(test_crate: &str, target_dir: impl AsRef<Path>) -> PathBuf {
    rustdoc_json_path_for_crate_impl(test_crate, Some(target_dir.as_ref()))
}

/// Helper to get the path to a freshly built rustdoc JSON file for the given
/// test-crate.
#[must_use]
fn rustdoc_json_path_for_crate_impl(test_crate: &str, target_dir: Option<&Path>) -> PathBuf {
    let mut builder = rustdoc_json::Builder::default()
        .toolchain("nightly".to_owned())
        .manifest_path(&format!("{}/Cargo.toml", test_crate))
        // The test framework is unable to capture output from child processes (see
        // https://users.rust-lang.org/t/cargo-doesnt-capture-stderr-in-tests/67045/4),
        // so build quietly to make running tests much less noisy
        .quiet(true);

    if let Some(target_dir) = target_dir {
        builder = builder.target_dir(target_dir);
    }

    builder.build().unwrap()
}

/// Adds `./target/debug` to `PATH` so that the subcommand `cargo public-api`
/// starts working (since `./target/debug` contains the `cargo-public-api`
/// binary).
pub fn add_target_debug_to_path() {
    let mut bin_dir = std::env::current_exe().unwrap(); // ".../target/debug/deps/cargo_public_api_bin_tests-d0f2f926b349fbb9"
    bin_dir.pop(); // Pop "cargo_public_api_bin_tests-d0f2f926b349fbb9"
    bin_dir.pop(); // Pop "deps"
    add_to_path(bin_dir); // ".../target/debug"
}

fn add_to_path(dir: PathBuf) {
    let mut path = std::env::var_os("PATH").unwrap();
    let mut dirs: Vec<_> = std::env::split_paths(&path).collect();
    dirs.insert(0, dir);
    path = std::env::join_paths(dirs).unwrap();
    std::env::set_var("PATH", path);
}

/// Installs a toolchain if it is not already installed.
pub fn ensure_toolchain_installed(toolchain: &str) {
    if !is_toolchain_installed(toolchain) {
        install_toolchain(toolchain);
    }
}

fn is_toolchain_installed(toolchain: &str) -> bool {
    std::process::Command::new("rustup")
        .arg("run")
        .arg(toolchain)
        .arg("cargo")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .unwrap()
        .success()
}

fn install_toolchain(toolchain: &str) {
    eprintln!("Installing toolchain {}", toolchain);
    std::process::Command::new("rustup")
        .arg("--quiet")
        .arg("toolchain")
        .arg("install")
        .arg("--no-self-update")
        .arg("--profile")
        .arg("minimal")
        .arg(toolchain)
        .unwrap();
}
