use pretty_assertions::assert_eq;
use public_items::Options;

#[test]
fn bat_v0_19_0() {
    assert_public_items(
        include_str!("./rustdoc_json/bat-v0.19.0.json"),
        include_str!("./rustdoc_json/bat-v0.19.0-expected.txt"),
    );
}

#[test]
fn public_items_git() {
    assert_public_items(
        include_str!("./rustdoc_json/public_items-git.json"),
        include_str!("./rustdoc_json/public_items-git-expected.txt"),
    );
}

#[test]
fn syntect_v4_6_0() {
    assert_public_items(
        include_str!("./rustdoc_json/syntect-v4.6.0.json"),
        include_str!("./rustdoc_json/syntect-v4.6.0-expected.txt"),
    );
}

#[test]
fn thiserror_v1_0_30() {
    assert_public_items(
        include_str!("./rustdoc_json/thiserror-1.0.30.json"),
        include_str!("./rustdoc_json/thiserror-1.0.30-expected.txt"),
    );
}

#[test]
fn public_items_git_with_blanket_implementations() {
    assert_public_items_with_blanket_implementations(
        include_str!("./rustdoc_json/public_items-git.json"),
        include_str!("./rustdoc_json/public_items-git-expected-with-blanket-implementations.txt"),
    );
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

    let actual: Vec<String> =
        public_items::sorted_public_items_from_rustdoc_json_str(rustdoc_json_str, options)
            .unwrap()
            .into_iter()
            .map(|x| format!("{}", x))
            .collect();

    let expected = expected_output_to_string_vec(expected_output);

    assert_eq!(expected, actual);
}

fn expected_output_to_string_vec(expected_output: &str) -> Vec<String> {
    expected_output.split('\n').map(String::from).collect()
}
