use public_api::MINIMUM_RUSTDOC_JSON_VERSION;

#[test]
fn installation_instructions_mentions_minimum_rustdoc_json_version() {
    let readme = include_str!("../README.md");
    let expected_installation_instruction = format!(
        "# Install {} or later so you can build up-to-date rustdoc JSON files",
        MINIMUM_RUSTDOC_JSON_VERSION
    );
    assert!(readme.contains(&expected_installation_instruction));
}
