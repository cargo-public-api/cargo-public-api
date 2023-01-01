// deny in CI, only warn here
#![warn(clippy::all, clippy::pedantic)]

use std::path::Path;

use expect_test::expect_file;
use public_api::{Error, Options, PublicApi};

use tempfile::tempdir;

mod common;
use common::rustdoc_json_path_for_crate;

#[test]
fn public_api() -> Result<(), Box<dyn std::error::Error>> {
    let rustdoc_json = rustdoc_json::Builder::default()
        .toolchain("nightly".to_owned())
        .build()?;

    let public_api = PublicApi::from_rustdoc_json(rustdoc_json, Options::default())?;

    expect_test::expect_file!["../public-api.txt"].assert_eq(&public_api.to_string());

    Ok(())
}

#[test]
fn not_simplified() {
    // Create independent build dir so all tests can run in parallel
    let build_dir = tempdir().unwrap();

    assert_public_api(
        rustdoc_json_path_for_crate("../test-apis/example_api-v0.2.0", &build_dir),
        "./expected-output/example_api-v0.2.0-not-simplified.txt",
        Options::default(),
    );
}

#[test]
fn diff_with_added_items() {
    // Create independent build dirs so all tests can run in parallel
    let build_dir = tempdir().unwrap();
    let build_dir2 = tempdir().unwrap();

    assert_public_api_diff(
        rustdoc_json_path_for_crate("../test-apis/example_api-v0.1.0", &build_dir),
        rustdoc_json_path_for_crate("../test-apis/example_api-v0.2.0", &build_dir2),
        "./expected-output/diff_with_added_items.txt",
    );
}

#[test]
fn no_diff() {
    // Create independent build dirs so all tests can run in parallel
    let build_dir = tempdir().unwrap();
    let build_dir2 = tempdir().unwrap();

    // No change to the public API
    assert_public_api_diff(
        rustdoc_json_path_for_crate("../test-apis/comprehensive_api", &build_dir),
        rustdoc_json_path_for_crate("../test-apis/comprehensive_api", &build_dir2),
        "./expected-output/no_diff.txt",
    );
}

#[test]
fn diff_with_removed_items() {
    // Create independent build dirs so all tests can run in parallel
    let build_dir = tempdir().unwrap();
    let build_dir2 = tempdir().unwrap();

    assert_public_api_diff(
        rustdoc_json_path_for_crate("../test-apis/example_api-v0.2.0", &build_dir2),
        rustdoc_json_path_for_crate("../test-apis/example_api-v0.1.0", &build_dir),
        "./expected-output/diff_with_removed_items.txt",
    );
}

#[test]
fn comprehensive_api() {
    // Create independent build dir so all tests can run in parallel
    let build_dir = tempdir().unwrap();

    assert_simplified_public_api(
        rustdoc_json_path_for_crate("../test-apis/comprehensive_api", &build_dir),
        "./expected-output/comprehensive_api.txt",
    );
}

#[test]
fn comprehensive_api_proc_macro() {
    // Create independent build dir so all tests can run in parallel
    let build_dir = tempdir().unwrap();

    assert_simplified_public_api(
        rustdoc_json_path_for_crate("../test-apis/comprehensive_api_proc_macro", &build_dir),
        "./expected-output/comprehensive_api_proc_macro.txt",
    );
}

#[test]
fn invalid_json() {
    let result = PublicApi::from_rustdoc_json_str("}}}}}}}}}", Options::default());
    assert!(matches!(result, Err(Error::SerdeJsonError(_))));
}

#[test]
fn options() {
    let options = Options::default();

    // If we don't do this, we will not have code coverage 100% of functions in
    // lib.rs, which is more annoying than doing this clone
    #[allow(clippy::clone_on_copy)]
    let _ = options.clone();
}

fn assert_public_api_diff(
    old_json: impl AsRef<Path>,
    new_json: impl AsRef<Path>,
    expected: impl AsRef<Path>,
) {
    let old = PublicApi::from_rustdoc_json(old_json, Options::default()).unwrap();
    let new = PublicApi::from_rustdoc_json(new_json, Options::default()).unwrap();

    let diff = public_api::diff::PublicApiDiff::between(old, new);
    expect_file![expected.as_ref()].assert_debug_eq(&diff);
}

/// Asserts that the public API of the crate in the given rustdoc JSON file
/// matches the expected output. For brevity, Auto Trait or Blanket impls are
/// not included.
fn assert_simplified_public_api(json: impl AsRef<Path>, expected: impl AsRef<Path>) {
    let mut options = Options::default();
    options.simplified = true;
    assert_public_api(json, expected, options);
}

/// Asserts that the public API of the crate in the given rustdoc JSON file
/// matches the expected output.
fn assert_public_api(
    rustdoc_json: impl AsRef<Path>,
    expected_output: impl AsRef<Path>,
    options: Options,
) {
    let api = PublicApi::from_rustdoc_json(rustdoc_json, options)
        .unwrap()
        .to_string();

    expect_file![expected_output.as_ref()].assert_eq(&api);
}
