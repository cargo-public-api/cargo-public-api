use pretty_assertions::assert_eq;

#[test]
fn syntect_v4_6_0() {
    assert_public_items(
        include_str!("./rustdoc_json/syntect-v4.6.0_FORMAT_VERSION_10.json"),
        include_str!("./rustdoc_json/syntect-v4.6.0-expected.txt"),
    );
}

fn assert_public_items(rustdoc_json_str: &str, expected_output: &str) {
    let actual = public_items::sorted_public_items_from_rustdoc_json_str(rustdoc_json_str).unwrap();

    let expected = expected_output_to_string_vec(expected_output);

    assert_eq!(expected, actual);
}

fn expected_output_to_string_vec(expected_output: &str) -> Vec<String> {
    expected_output.split('\n').map(String::from).collect()
}
