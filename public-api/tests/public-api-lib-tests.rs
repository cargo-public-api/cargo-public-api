// deny in CI, only warn here
#![warn(clippy::all)]

use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use expect_test::expect_file;
use public_api::Error;

use tempfile::{tempdir, NamedTempFile, TempDir};

mod common;
use common::{
    builder_for_crate, rustdoc_json_path_for_crate, rustdoc_json_path_for_temp_crate,
    simplified_builder_for_crate,
};

#[test]
fn public_api() -> Result<(), Box<dyn std::error::Error>> {
    let rustdoc_json = rustdoc_json::Builder::default()
        .toolchain("nightly")
        .build()?;

    let public_api = public_api::Builder::from_rustdoc_json(rustdoc_json).build()?;

    expect_test::expect_file!["public-api.txt"].assert_eq(&public_api.to_string());

    Ok(())
}

#[test]
fn not_simplified() {
    // Create independent build dir so all tests can run in parallel
    let build_dir = tempdir().unwrap();

    assert_public_api(
        builder_for_crate("../test-apis/example_api-v0.2.0", &build_dir),
        "./expected-output/example_api-v0.2.0-not-simplified.txt",
    );
}

#[test]
fn simplified_without_auto_derived_impls() {
    // Create independent build dir so all tests can run in parallel
    let build_dir = tempdir().unwrap();

    assert_public_api(
        builder_for_crate("../test-apis/example_api-v0.2.0", &build_dir)
            .omit_auto_derived_impls(true)
            .omit_blanket_impls(true)
            .omit_auto_trait_impls(true),
        "./expected-output/example_api-v0.2.0-simplified_without_auto_derived_impls.txt",
    );
}

#[test]
fn omit_blanket_impls() {
    // Create independent build dir so all tests can run in parallel
    let build_dir = tempdir().unwrap();

    assert_public_api(
        builder_for_crate("../test-apis/example_api-v0.2.0", &build_dir).omit_blanket_impls(true),
        "./expected-output/example_api-v0.2.0-omit_blanket_impls.txt",
    );
}

#[test]
fn omit_auto_trait_impls() {
    // Create independent build dir so all tests can run in parallel
    let build_dir = tempdir().unwrap();

    assert_public_api(
        builder_for_crate("../test-apis/example_api-v0.2.0", &build_dir)
            .omit_auto_trait_impls(true),
        "./expected-output/example_api-v0.2.0-omit_auto_trait_impls.txt",
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
        v1.json_path,
        v2.json_path,
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

    assert_public_api(
        simplified_builder_for_crate("../test-apis/comprehensive_api", &build_dir),
        "./expected-output/comprehensive_api.txt",
    );
}

#[test]
fn comprehensive_api_proc_macro() {
    // Create independent build dir so all tests can run in parallel
    let build_dir = tempdir().unwrap();

    assert_public_api(
        simplified_builder_for_crate("../test-apis/comprehensive_api_proc_macro", &build_dir),
        "./expected-output/comprehensive_api_proc_macro.txt",
    );
}

#[test]
fn auto_traits() {
    // Create independent build dir so all tests can run in parallel
    let build_dir = tempdir().unwrap();

    assert_public_api(
        builder_for_crate("../test-apis/auto_traits", &build_dir).omit_blanket_impls(true),
        "./expected-output/auto_traits.txt",
    );
}

/// Test that `debug_sorting` does not result in stack overflow because of
/// recursion. This can quite easily happen unless we test for it continuously.
/// We don't care what the exact output is, just that we don't crash.
#[test]
fn comprehensive_api_debug_sorting_no_stack_overflow() {
    // Create independent build dir so all tests can run in parallel
    let build_dir = tempdir().unwrap();

    let rustdoc_json = rustdoc_json_path_for_crate("../test-apis/comprehensive_api", &build_dir);
    let _api = public_api::Builder::from_rustdoc_json(rustdoc_json)
        .debug_sorting(true)
        .build()
        .unwrap()
        .to_string();
}

#[test]
fn invalid_json() {
    let invalid_json = NamedTempFile::new().unwrap();
    write!(invalid_json.as_file(), "}}}}}}}}}}").unwrap();
    let result = public_api::Builder::from_rustdoc_json(invalid_json.path()).build();
    assert!(matches!(result, Err(Error::SerdeJsonError(_))));
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
    old_json: impl Into<PathBuf>,
    new_json: impl Into<PathBuf>,
    expected: impl AsRef<Path>,
) {
    let old = public_api::Builder::from_rustdoc_json(old_json)
        .build()
        .unwrap();
    let new = public_api::Builder::from_rustdoc_json(new_json)
        .build()
        .unwrap();

    let diff = public_api::diff::PublicApiDiff::between(old, new);
    expect_file![expected.as_ref()].assert_debug_eq(&diff);
}

/// Asserts that the public API of the crate in the given rustdoc JSON file
/// matches the expected output.
fn assert_public_api(builder: public_api::Builder, expected_output: impl AsRef<Path>) {
    let api = builder.build().unwrap().to_string();

    expect_file![expected_output.as_ref()].assert_eq(&api);
}
