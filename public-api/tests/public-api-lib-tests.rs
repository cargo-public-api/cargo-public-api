// deny in CI, only warn here
#![warn(clippy::all)]

use std::{fs, io::Write, path::PathBuf};

use public_api::Error;

use pretty_assertions::assert_eq;

use tempfile::{NamedTempFile, TempDir, tempdir};

mod common;
use common::{
    builder_for_crate, rustdoc_json_path_for_crate, rustdoc_json_path_for_temp_crate,
    simplified_builder_for_crate,
};

#[test]
fn public_api() -> Result<(), Box<dyn std::error::Error>> {
    public_api_for_manifest("public-api", "Cargo.toml")
}

// To avoid circular workspace dependencies we test the API surface of this
// crate as well.
#[test]
fn public_api_for_rustup_toolchain() -> Result<(), Box<dyn std::error::Error>> {
    public_api_for_manifest("rustup-toolchain", "../rustup-toolchain/Cargo.toml")
}

// To avoid circular workspace dependencies we test the API surface of this
// crate as well.
#[test]
fn public_api_for_rustdoc_json() -> Result<(), Box<dyn std::error::Error>> {
    public_api_for_manifest("rustdoc-json", "../rustdoc-json/Cargo.toml")
}

fn public_api_for_manifest(
    snapshot_name: &str,
    manifest_path: impl AsRef<std::path::Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    let rustdoc_json = rustdoc_json::Builder::default()
        .toolchain("nightly")
        .manifest_path(&manifest_path)
        .build()?;

    let public_api = public_api::Builder::from_rustdoc_json(rustdoc_json).build()?;

    assert_or_bless::assert_eq_or_bless_if(
        public_api.to_string(),
        format!("{snapshot_name}.txt"),
        std::env::var("PUBLIC_API_BLESS")
            .map_or(false, |s| s == "1" || s == "yes" || s == "true"),
    );

    Ok(())
}

#[test]
fn not_simplified() {
    // Create independent build dir so all tests can run in parallel
    let build_dir = tempdir().unwrap();

    assert_public_api(
        builder_for_crate("../test-apis/example_api-v0.2.0", &build_dir),
        "example_api-v0.2.0-not-simplified",
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
        "example_api-v0.2.0-simplified_without_auto_derived_impls",
    );
}

#[test]
fn omit_blanket_impls() {
    // Create independent build dir so all tests can run in parallel
    let build_dir = tempdir().unwrap();

    assert_public_api(
        builder_for_crate("../test-apis/example_api-v0.2.0", &build_dir).omit_blanket_impls(true),
        "example_api-v0.2.0-omit_blanket_impls",
    );
}

#[test]
fn omit_auto_trait_impls() {
    // Create independent build dir so all tests can run in parallel
    let build_dir = tempdir().unwrap();

    assert_public_api(
        builder_for_crate("../test-apis/example_api-v0.2.0", &build_dir)
            .omit_auto_trait_impls(true),
        "example_api-v0.2.0-omit_auto_trait_impls",
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
        "diff_with_added_items",
    );
}

#[test]
fn empty_diff() {
    // Create independent build dirs so all tests can run in parallel
    let build_dir = tempdir().unwrap();
    let build_dir2 = tempdir().unwrap();

    // No change to the public API
    assert_public_api_diff(
        rustdoc_json_path_for_crate("../test-apis/comprehensive_api", &build_dir),
        rustdoc_json_path_for_crate("../test-apis/comprehensive_api", &build_dir2),
        "empty_diff",
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
        "diff_move_item_between_inherent_impls",
    );
}

#[test]
fn diff_empty_when_changing_inherent_impl_to_auto_derived_impl() {
    let v1 = rustdoc_json_for_lib(
        r#"
#[derive(Clone)]
pub enum Foo {
    A,
    B,
}

impl Default for Foo {
    fn default() -> Foo {
        Foo::A
    }
}
    "#,
    );

    let v2 = rustdoc_json_for_lib(
        r#"
#[derive(Clone, Default)]
pub enum Foo {
    #[default]
    A,
    B,
}
    "#,
    );

    // Note: The test will pass with assert_public_api_diff() because it uses
    // [`public_api::diff::PublicApiDiff::between`] which is smart enough to
    // realize there is no diff. But we also want to make sure the above case
    // does not result in any textual diff either.
    assert_no_textual_public_api_diff(v1.json_path, v2.json_path);
}

#[test]
fn diff_with_removed_items() {
    // Create independent build dirs so all tests can run in parallel
    let build_dir = tempdir().unwrap();
    let build_dir2 = tempdir().unwrap();

    assert_public_api_diff(
        rustdoc_json_path_for_crate("../test-apis/example_api-v0.2.0", &build_dir2),
        rustdoc_json_path_for_crate("../test-apis/example_api-v0.1.0", &build_dir),
        "diff_with_removed_items",
    );
}

#[test]
fn comprehensive_api() {
    // Create independent build dir so all tests can run in parallel
    let build_dir = tempdir().unwrap();

    assert_public_api(
        simplified_builder_for_crate("../test-apis/comprehensive_api", &build_dir),
        "comprehensive_api",
    );
}

#[test]
fn comprehensive_api_proc_macro() {
    // Create independent build dir so all tests can run in parallel
    let build_dir = tempdir().unwrap();

    assert_public_api(
        simplified_builder_for_crate("../test-apis/comprehensive_api_proc_macro", &build_dir),
        "comprehensive_api_proc_macro",
    );
}

#[test]
fn auto_traits() {
    // Create independent build dir so all tests can run in parallel
    let build_dir = tempdir().unwrap();

    assert_public_api(
        builder_for_crate("../test-apis/auto_traits", &build_dir).omit_blanket_impls(true),
        "auto_traits",
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

#[test]
fn find_function() {
    use rustdoc_types::{Crate, Id, ItemEnum};

    fn is_function(krate: &Crate, id: Id) -> bool {
        matches!(krate.index.get(&id).unwrap().inner, ItemEnum::Function(_))
    }

    let json = rustdoc_json_for_lib(
        r#"
pub mod a_mod {
    pub fn a_function() {
    }
}
    "#,
    );

    let public_api = public_api::Builder::from_rustdoc_json(&json.json_path)
        .build()
        .unwrap();

    let file = fs::File::open(json.json_path).unwrap();
    let krate = serde_json::from_reader::<_, Crate>(file).unwrap();

    let public_item = public_api
        .items()
        .find(|public_item| is_function(&krate, public_item.id()))
        .unwrap();

    let id = public_item.id();
    let item = krate.index.get(&id).unwrap();
    assert_eq!(Some("a_function"), item.name.as_deref());

    let parent_id = public_item.parent_id().unwrap();
    let parent_item = krate.index.get(&parent_id).unwrap();
    assert!(matches!(parent_item.inner, ItemEnum::Module(_)));
    assert_eq!(Some("a_mod"), parent_item.name.as_deref());
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
    test_name: &str,
) {
    let old = public_api::Builder::from_rustdoc_json(old_json)
        .build()
        .unwrap();
    let new = public_api::Builder::from_rustdoc_json(new_json)
        .build()
        .unwrap();

    let diff = public_api::diff::PublicApiDiff::between(old, new);
    insta::assert_debug_snapshot!(test_name, diff);
}

// PublicApiDiff::between() is smarter than a textual diff, but in some cases we
// still care about a textual diff. When we care about the textual diff, we use
// this function.
fn assert_no_textual_public_api_diff(old_json: impl Into<PathBuf>, new_json: impl Into<PathBuf>) {
    let old = public_api::Builder::from_rustdoc_json(old_json)
        .build()
        .unwrap()
        .to_string();
    let new = public_api::Builder::from_rustdoc_json(new_json)
        .build()
        .unwrap()
        .to_string();

    assert_eq!(new, old);
}

/// Asserts that the public API of the crate in the given rustdoc JSON file
/// matches the expected output.
fn assert_public_api(builder: public_api::Builder, test_name: &str) {
    let api = builder.build().unwrap().to_string();

    insta::assert_snapshot!(test_name, api);
}
