use pretty_assertions::assert_eq;

#[test]
fn syntect_v4_6_0() {
    assert_public_items(
        include_str!("./rustdoc_json/syntect-v4.6.0_FORMAT_VERSION_10.json"),
        include_str!("./rustdoc_json/syntect-v4.6.0-expected.txt"),
    );
}

#[test]
fn thiserror_v1_0_30() {
    assert_public_items(
        include_str!("./rustdoc_json/thiserror-v1.0.30_FORMAT_VERSION_10.json"),
        include_str!("./rustdoc_json/thiserror-v1.0.30-expected.txt"),
    );
}

fn assert_public_items(rustdoc_json_str: &str, expected_output: &str) {
    let actual: Vec<String> =
        public_items::sorted_public_items_from_rustdoc_json_str(rustdoc_json_str)
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
