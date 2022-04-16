use std::{fmt::Display, path::Path};

use pretty_assertions::assert_eq;
use public_items::{public_items_from_rustdoc_json_str, Error, Options};

struct ExpectedDiff<'a> {
    removed: &'a [&'a str],
    changed: &'a [(&'a str, &'a str)],
    added: &'a [&'a str],
}

#[test]
fn public_items_v0_4_0_with_blanket_implementations() {
    assert_public_items_with_blanket_implementations(
        include_str!("./rustdoc_json/public_items-v0.4.0.json"),
        include_str!("./expected_output/public_items-v0.4.0-with-blanket-implementations.txt"),
    );
}

#[test]
fn public_items_diff_between_v0_0_4_and_v0_0_5() {
    assert_public_items_diff(
        include_str!("./rustdoc_json/public_items-v0.0.4.json"),
        include_str!("./rustdoc_json/public_items-v0.0.5.json"),
        &ExpectedDiff {
            removed: &["pub fn public_items::from_rustdoc_json_str(rustdoc_json_str: &str) -> Result<HashSet<String>>"],
            changed: &[],
            added: &["pub fn public_items::sorted_public_items_from_rustdoc_json_str(rustdoc_json_str: &str) -> Result<Vec<String>>"],
        }
    );
}

#[test]
fn public_items_diff_between_v0_2_0_and_v0_3_0() {
    // No change to the public API
    assert_public_items_diff(
        include_str!("./rustdoc_json/public_items-v0.2.0.json"),
        include_str!("./rustdoc_json/public_items-v0.3.0.json"),
        &ExpectedDiff {
            removed: &[],
            changed: &[],
            added: &[],
        },
    );
}

#[test]
fn public_items_diff_between_v0_3_0_and_v0_4_0() {
    assert_public_items_diff(
        include_str!("./rustdoc_json/public_items-v0.3.0.json"),
        include_str!("./rustdoc_json/public_items-v0.4.0.json"),
        &ExpectedDiff {
            removed: &[],
            changed: &[
                (
                    "pub fn public_items::sorted_public_items_from_rustdoc_json_str(rustdoc_json_str: &str) -> Result<Vec<PublicItem>>",
                    "pub fn public_items::sorted_public_items_from_rustdoc_json_str(rustdoc_json_str: &str, options: Options) -> Result<Vec<PublicItem>>",
                )
            ],
            added: &[
                  "pub fn public_items::Options::clone(&self) -> Options",
                  "pub fn public_items::Options::default() -> Self",
                  "pub fn public_items::Options::fmt(&self, f: &mut $crate::fmt::Formatter<'_>) -> $crate::fmt::Result",
                  "pub struct field public_items::Options::with_blanket_implementations: bool",
                  "pub struct public_items::Options",
                ],
        },
    );
}

#[test]
fn comprehensive_api() {
    assert_public_items(
        &rustdoc_json("comprehensive_api"),
        include_str!("./expected_output/comprehensive_api.txt"),
    );
}

#[test]
fn comprehensive_api_proc_macro() {
    assert_public_items(
        &rustdoc_json("comprehensive_api_proc_macro"),
        include_str!("./expected_output/comprehensive_api_proc_macro.txt"),
    );
}

/// I confess: this test is mainly to get function code coverage on Ord
#[test]
fn public_item_ord() {
    let public_items =
        public_items_from_rustdoc_json_str(&rustdoc_json("comprehensive_api"), Options::default())
            .unwrap();

    let generic_arg = public_items
        .clone()
        .into_iter()
        .find(|x| format!("{}", x).contains("generic_arg"))
        .unwrap();

    let generic_bound = public_items
        .into_iter()
        .find(|x| format!("{}", x).contains("generic_bound"))
        .unwrap();

    assert_eq!(generic_arg.max(generic_bound.clone()), generic_bound);
}

#[test]
fn invalid_json() {
    let result = public_items_from_rustdoc_json_str("}}}}}}}}}", Options::default());
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

#[test]
fn pretty_printed_diff() {
    let options = Options::default();
    let old = public_items_from_rustdoc_json_str(
        include_str!("./rustdoc_json/public_items-v0.2.0.json"),
        options,
    )
    .unwrap();
    let new = public_items_from_rustdoc_json_str(
        include_str!("./rustdoc_json/public_items-v0.4.0.json"),
        options,
    )
    .unwrap();

    let diff = public_items::diff::PublicItemsDiff::between(old, new);
    let pretty_printed = format!("{:#?}", diff);
    assert_eq!(pretty_printed, "PublicItemsDiff {
    removed: [],
    changed: [
        ChangedPublicItem {
            old: pub fn public_items::sorted_public_items_from_rustdoc_json_str(rustdoc_json_str: &str) -> Result<Vec<PublicItem>>,
            new: pub fn public_items::sorted_public_items_from_rustdoc_json_str(rustdoc_json_str: &str, options: Options) -> Result<Vec<PublicItem>>,
        },
    ],
    added: [
        pub fn public_items::Options::clone(&self) -> Options,
        pub fn public_items::Options::default() -> Self,
        pub fn public_items::Options::fmt(&self, f: &mut $crate::fmt::Formatter<'_>) -> $crate::fmt::Result,
        pub struct field public_items::Options::with_blanket_implementations: bool,
        pub struct public_items::Options,
    ],
}");
}

/// Synchronously generate the rustdoc JSON for a library crate.
fn build_rustdoc_json<P: AsRef<Path>>(manifest_path: P) {
    let mut command = std::process::Command::new("cargo");
    command.args(["+nightly", "doc", "--lib", "--no-deps"]);
    command.arg("--manifest-path");
    command.arg(manifest_path.as_ref());
    command.env("RUSTDOCFLAGS", "-Z unstable-options --output-format json");
    assert!(command.spawn().unwrap().wait().unwrap().success());
}

/// Helper to get a [String] that contains the rustdoc JSON (freshly built) for
/// our in-repo `comprehensive_api` test crate.
///
/// The easiest way to explore the API for a human is by running
/// ```bash
/// cargo doc --manifest-path ./tests/crates/comprehensive_api/Cargo.toml --open
/// ```
fn rustdoc_json(test_crate: &str) -> String {
    build_rustdoc_json(format!("./tests/crates/{}/Cargo.toml", test_crate));
    std::fs::read_to_string(format!("./target/doc/{}.json", test_crate)).unwrap()
}

fn assert_public_items_diff(old_json: &str, new_json: &str, expected: &ExpectedDiff) {
    let old = public_items_from_rustdoc_json_str(old_json, Options::default()).unwrap();
    let new = public_items_from_rustdoc_json_str(new_json, Options::default()).unwrap();

    let diff = public_items::diff::PublicItemsDiff::between(old, new);

    assert_eq!(expected.added, into_strings(diff.added));
    assert_eq!(expected.removed, into_strings(diff.removed));

    let expected_changed: Vec<_> = expected
        .changed
        .iter()
        .map(|x| (x.0.to_owned(), x.1.to_owned()))
        .collect();
    let actual_changed: Vec<_> = diff
        .changed
        .iter()
        .map(|x| (format!("{}", &x.old), format!("{}", &x.new)))
        .collect();
    assert_eq!(expected_changed, actual_changed);
}

fn assert_public_items(json: &str, expected: &str) {
    assert_public_items_impl(json, expected, false);
}

fn assert_public_items_with_blanket_implementations(json: &str, expected: &str) {
    assert_public_items_impl(json, expected, true);
}

fn assert_public_items_impl(
    rustdoc_json_str: &str,
    expected_output: &str,
    with_blanket_implementations: bool,
) {
    let mut options = Options::default();
    options.with_blanket_implementations = with_blanket_implementations;
    options.sorted = true;

    let actual =
        into_strings(public_items_from_rustdoc_json_str(rustdoc_json_str, options).unwrap());

    let expected = expected_output_to_string_vec(expected_output);

    assert_eq!(expected, actual);
}

fn expected_output_to_string_vec(expected_output: &str) -> Vec<String> {
    expected_output
        .split('\n')
        .map(String::from)
        .filter(|s| !s.is_empty()) // Remove empty entry caused by trailing newline in files
        .collect()
}

fn into_strings(items: Vec<impl Display>) -> Vec<String> {
    items.into_iter().map(|x| format!("{}", x)).collect()
}

/// To be honest this is mostly to get higher code coverage numbers.
/// But it is actually useful thing to test.
fn ensure_impl_debug(impl_debug: &impl std::fmt::Debug) {
    eprintln!("Yes, this can be debugged: {:?}", impl_debug);
}
