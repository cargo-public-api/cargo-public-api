use std::fmt::Display;

use pretty_assertions::assert_eq;
use public_items::{public_items_from_rustdoc_json_str, Options};

struct ExpectedDiff<'a> {
    removed: &'a [&'a str],
    changed: &'a [(&'a str, &'a str)],
    added: &'a [&'a str],
}

#[test]
fn bat_v0_19_0() {
    assert_public_items(
        include_str!("./rustdoc_json/bat-v0.19.0.json"),
        include_str!("./expected_output/bat-v0.19.0.txt"),
    );
}

#[test]
fn syntect_v4_6_0() {
    assert_public_items(
        include_str!("./rustdoc_json/syntect-v4.6.0.json"),
        include_str!("./expected_output/syntect-v4.6.0.txt"),
    );
}

#[test]
fn thiserror_v1_0_30() {
    assert_public_items(
        include_str!("./rustdoc_json/thiserror-1.0.30.json"),
        include_str!("./expected_output/thiserror-1.0.30.txt"),
    );
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
                  "pub struct public_items::Options",
                  "pub struct field public_items::Options::with_blanket_implementations: bool",
                ],
        },
    );
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
    expected_output.split('\n').map(String::from).collect()
}

fn into_strings(items: Vec<impl Display>) -> Vec<String> {
    items.into_iter().map(|x| format!("{}", x)).collect()
}
