// deny in CI, only warn here
#![warn(clippy::all, clippy::pedantic)]

use std::{fmt::Write, path::Path};

use expect_test::expect_file;
use public_api::{Error, Options, PublicApi};

// rust-analyzer bug: https://github.com/rust-lang/rust-analyzer/issues/9173
#[path = "../../test-utils/src/lib.rs"]
mod test_utils;
use tempfile::tempdir;
use test_utils::rustdoc_json_path_for_crate;

#[test]
fn not_simplified() {
    // Create independent build dir so all tests can run in parallel
    let build_dir = tempdir().unwrap();

    assert_public_api_not_simplified(
        rustdoc_json_path_for_crate("../test-apis/example_api-v0.2.0", &build_dir),
        "./expected-output/example_api-v0.2.0-not-simplified.txt",
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

    assert_public_api(
        rustdoc_json_path_for_crate("../test-apis/comprehensive_api", &build_dir),
        "./expected-output/comprehensive_api.txt",
    );
}

#[test]
fn comprehensive_api_proc_macro() {
    // Create independent build dir so all tests can run in parallel
    let build_dir = tempdir().unwrap();

    assert_public_api(
        rustdoc_json_path_for_crate("../test-apis/comprehensive_api_proc_macro", &build_dir),
        "./expected-output/comprehensive_api_proc_macro.txt",
    );
}

#[test]
fn invalid_json() {
    let result = PublicApi::from_rustdoc_json_str("}}}}}}}}}", Options::default());
    ensure_impl_debug(&result);
    assert!(matches!(result, Err(Error::SerdeJsonError(_))));
}

#[test]
fn options() {
    let options = Options::default();
    ensure_impl_debug(&options);

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

fn assert_public_api(json: impl AsRef<Path>, expected: impl AsRef<Path>) {
    assert_public_api_impl(json, expected, true);
}

fn assert_public_api_not_simplified(json: impl AsRef<Path>, expected: impl AsRef<Path>) {
    assert_public_api_impl(json, expected, false);
}

fn assert_public_api_impl(
    rustdoc_json: impl AsRef<Path>,
    expected_output: impl AsRef<Path>,
    simplified: bool,
) {
    let mut options = Options::default();
    options.simplified = simplified;
    options.sorted = true;

    let api = PublicApi::from_rustdoc_json(rustdoc_json, options).unwrap();

    let mut actual = String::new();
    for item in api.items() {
        writeln!(&mut actual, "{}", item).unwrap();
    }

    expect_file![expected_output.as_ref()].assert_eq(&actual);
}

/// To be honest this is mostly to get higher code coverage numbers.
/// But it is actually useful thing to test.
fn ensure_impl_debug(impl_debug: &impl std::fmt::Debug) {
    eprintln!("Yes, this can be debugged: {:?}", impl_debug);
}
