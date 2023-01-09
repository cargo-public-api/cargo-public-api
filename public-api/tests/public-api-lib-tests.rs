// deny in CI, only warn here
#![warn(clippy::all, clippy::pedantic)]

use std::{
    fs,
    path::{Path, PathBuf},
};

use expect_test::expect_file;
use public_api::{Error, Options, PublicApi};

use tempfile::{tempdir, TempDir};

mod common;
use common::{rustdoc_json_path_for_crate, rustdoc_json_path_for_temp_crate};

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
fn simplified_without_auto_derived_impls() {
    // Create independent build dir so all tests can run in parallel
    let build_dir = tempdir().unwrap();

    let mut options = simplified();
    options.omit_auto_derived_impls = true;

    assert_public_api(
        rustdoc_json_path_for_crate("../test-apis/example_api-v0.2.0", &build_dir),
        "./expected-output/example_api-v0.2.0-simplified_without_auto_derived_impls.txt",
        options,
    );
}

#[test]
fn omit_blanket_impls() {
    // Create independent build dir so all tests can run in parallel
    let build_dir = tempdir().unwrap();

    let mut options = Options::default();
    options.omit_blanket_impls = true;

    assert_public_api(
        rustdoc_json_path_for_crate("../test-apis/example_api-v0.2.0", &build_dir),
        "./expected-output/example_api-v0.2.0-omit_blanket_impls.txt",
        options,
    );
}

#[test]
fn omit_auto_trait_impls() {
    // Create independent build dir so all tests can run in parallel
    let build_dir = tempdir().unwrap();

    let mut options = Options::default();
    options.omit_auto_trait_impls = true;

    assert_public_api(
        rustdoc_json_path_for_crate("../test-apis/example_api-v0.2.0", &build_dir),
        "./expected-output/example_api-v0.2.0-omit_auto_trait_impls.txt",
        options,
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
fn diff_empty_when_item_moved_between_inherent_impls() {
    let v1 = rustdoc_json_for_lib(
        r#"
pub struct Foo;
impl Foo {
    pub fn f1() {}
    pub fn moved() {}
}
impl Foo {
    pub fn f2() {}
}
    "#,
    );

    let v2 = rustdoc_json_for_lib(
        r#"
pub struct Foo;
impl Foo {
    pub fn f1() {}
}
impl Foo {
    pub fn f2() {}
    pub fn moved() {}
}
    "#,
    );

    assert_public_api_diff(
        &v1.json_path,
        &v2.json_path,
        "./expected-output/diff_move_item_between_inherent_impls.txt",
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

/// Test that `debug_sorting` does not result in stack overflow because of
/// recursion. This can quite easily happen unless we test for it continuously.
/// We don't care what the exact output is, just that we don't crash.
#[test]
fn comprehensive_api_debug_sorting_no_stack_overflow() {
    // Create independent build dir so all tests can run in parallel
    let build_dir = tempdir().unwrap();

    let mut options = Options::default();
    options.debug_sorting = true;
    let rustdoc_json = rustdoc_json_path_for_crate("../test-apis/comprehensive_api", &build_dir);
    let _api = PublicApi::from_rustdoc_json(rustdoc_json, options)
        .unwrap()
        .to_string();
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

struct LibWithJson {
    json_path: PathBuf,

    /// Keep the tempdir alive so the json file doesn't get deleted
    _root: TempDir,
}

fn rustdoc_json_for_lib(lib: &str) -> LibWithJson {
    let root = tempdir().unwrap();

    let write = |file: &str, content: &str| {
        let file_path = root.path().join(file);
        fs::write(file_path, content).unwrap();
    };

    write(
        "Cargo.toml",
        "\
        [package]\n\
        name = \"lib\"\n\
        version = \"0.1.0\"\n\
        edition = \"2021\"\n\
        [lib]\n\
        path = \"lib.rs\"\n\
        ",
    );

    write("lib.rs", lib);

    LibWithJson {
        json_path: rustdoc_json_path_for_temp_crate(&root),
        _root: root,
    }
}

fn assert_public_api_diff(
    old_json: impl AsRef<Path>,
    new_json: impl AsRef<Path>,
    expected: impl AsRef<Path>,
) {
    let options = Options::default();
    let old = PublicApi::from_rustdoc_json(old_json, options).unwrap();
    let new = PublicApi::from_rustdoc_json(new_json, options).unwrap();

    let diff = public_api::diff::PublicApiDiff::between(old, new);
    expect_file![expected.as_ref()].assert_debug_eq(&diff);
}

/// Asserts that the public API of the crate in the given rustdoc JSON file
/// matches the expected output. For brevity, Auto Trait or Blanket impls are
/// not included.
fn assert_simplified_public_api(json: impl AsRef<Path>, expected: impl AsRef<Path>) {
    assert_public_api(json, expected, simplified());
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

/// Returns options for a so called "simplified" API, which is an API without
/// Auto Trait or Blanket impls, to reduce public item noise.
fn simplified() -> Options {
    let mut options = Options::default();
    options.omit_blanket_impls = true;
    options.omit_auto_trait_impls = true;
    options
}
