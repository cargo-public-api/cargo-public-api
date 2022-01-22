use pretty_assertions::assert_eq;

#[test]
fn syntect_v4_6_0() {
    assert_public_items(
        include_str!("./rustdoc_json/syntect-v4.6.0-by-rust-nightly-2021-11-15.json"),
        "./tests/rustdoc_json/syntect-v4.6.0-expected.txt",
    );
}

fn assert_public_items(rustdoc_json_str: &str, expected: &str) {
    let mut actual = public_items::public_items_from_rustdoc_json_str(rustdoc_json_str)
        .unwrap()
        .into_iter()
        .map(|i| format!("{}", i))
        .collect::<Vec<_>>();
    actual.sort();

    let expected = string_hash_set_from_str_array(expected);

    assert_eq!(actual, expected);
}

fn string_hash_set_from_str_array(path: &str) -> Vec<String> {
    std::fs::read_to_string(path)
        .unwrap()
        .split('\n')
        .map(String::from)
        .collect()
}
