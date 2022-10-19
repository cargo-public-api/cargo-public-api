// deny in CI, only warn here
#![warn(clippy::all, clippy::pedantic)]

use std::{fmt::Write, path::Path};

use public_api::{Error, Options, PublicApi};

// rust-analyzer bug: https://github.com/rust-lang/rust-analyzer/issues/9173
#[path = "../../test-utils/src/lib.rs"]
mod test_utils;
use test_utils::assert_eq_or_bless;
use test_utils::rustdoc_json_str_for_crate;

#[test]
fn with_blanket_implementations() {
    if std::env::var("BLESS").is_ok() {
        return; // To not race with include_str!()
    }

    assert_public_api_with_blanket_implementations(
        &rustdoc_json_str_for_crate("../test-apis/example_api-v0.2.0"),
        "./tests/expected-output/example_api-v0.2.0-with-blanket-implementations.txt",
    );
}

#[test]
fn diff_with_added_items() {
    assert_public_api_diff(
        &rustdoc_json_str_for_crate("../test-apis/example_api-v0.1.0"),
        &rustdoc_json_str_for_crate("../test-apis/example_api-v0.2.0"),
        "./tests/expected-output/diff_with_added_items.txt",
    );
}

#[test]
fn no_diff() {
    // No change to the public API
    assert_public_api_diff(
        &rustdoc_json_str_for_crate("../test-apis/comprehensive_api"),
        &rustdoc_json_str_for_crate("../test-apis/comprehensive_api"),
        "./tests/expected-output/no_diff.txt",
    );
}

#[test]
fn diff_with_removed_items() {
    assert_public_api_diff(
        &rustdoc_json_str_for_crate("../test-apis/example_api-v0.2.0"),
        &rustdoc_json_str_for_crate("../test-apis/example_api-v0.1.0"),
        "./tests/expected-output/diff_with_removed_items.txt",
    );
}

#[test]
fn comprehensive_api() {
    assert_public_api(
        &rustdoc_json_str_for_crate("../test-apis/comprehensive_api"),
        "./tests/expected-output/comprehensive_api.txt",
    );
}

#[test]
fn comprehensive_api_proc_macro() {
    assert_public_api(
        &rustdoc_json_str_for_crate("../test-apis/comprehensive_api_proc_macro"),
        "./tests/expected-output/comprehensive_api_proc_macro.txt",
    );
}

/// I confess: this test is mainly to get function code coverage on Ord
#[test]
fn public_item_ord() {
    let public_api = PublicApi::from_rustdoc_json_str(
        &rustdoc_json_str_for_crate("../test-apis/comprehensive_api"),
        Options::default(),
    )
    .unwrap();

    let generic_arg = public_api
        .items
        .clone()
        .into_iter()
        .find(|x| format!("{}", x).contains("generic_arg"))
        .unwrap();

    let generic_bound = public_api
        .items
        .into_iter()
        .find(|x| format!("{}", x).contains("generic_bound"))
        .unwrap();

    assert_eq!(generic_arg.max(generic_bound.clone()), generic_bound);
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

fn assert_public_api_diff(old_json: &str, new_json: &str, expected: impl AsRef<Path>) {
    let old = PublicApi::from_rustdoc_json_str(old_json, Options::default()).unwrap();
    let new = PublicApi::from_rustdoc_json_str(new_json, Options::default()).unwrap();

    let diff = public_api::diff::PublicApiDiff::between(old, new);
    let pretty_printed = format!("{:#?}", diff);
    assert_eq_or_bless(&pretty_printed, expected);
}

fn assert_public_api(json: &str, expected: impl AsRef<Path>) {
    assert_public_api_impl(json, expected, false);
}

fn assert_public_api_with_blanket_implementations(json: &str, expected: impl AsRef<Path>) {
    assert_public_api_impl(json, expected, true);
}

fn assert_public_api_impl(
    rustdoc_json_str: &str,
    expected_output: impl AsRef<Path>,
    with_blanket_implementations: bool,
) {
    let mut options = Options::default();
    options.with_blanket_implementations = with_blanket_implementations;
    options.sorted = true;

    let api = PublicApi::from_rustdoc_json_str(rustdoc_json_str, options).unwrap();

    let mut actual = String::new();
    for item in api.items {
        writeln!(&mut actual, "{}", item).unwrap();
    }

    assert_eq_or_bless(&actual, expected_output);
}

/// To be honest this is mostly to get higher code coverage numbers.
/// But it is actually useful thing to test.
fn ensure_impl_debug(impl_debug: &impl std::fmt::Debug) {
    eprintln!("Yes, this can be debugged: {:?}", impl_debug);
}
