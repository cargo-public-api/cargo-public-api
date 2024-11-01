//! See `docs/RELEASE.md` for information on how to use this.

fn main() {
    insert_file_contents_in_between(
        "README.md",
        "# Compatibility Matrix

| Version          | Understands the rustdoc JSON output of  |
| ---------------- | --------------------------------------- |
",
        &release_helper::compatibility_matrix::render(release_helper::version_info::TABLE),
        "| earlier versions | see [here]",
    );

    insert_file_contents_in_between(
        "README.md",
        "```sh
cargo +stable install cargo-public-api --locked
```

Ensure **",
        release_helper::version_info::TABLE[0].min_nightly_rust_version,
        "** or later is installed (does not need to be the active toolchain)",
    );

    insert_file_contents_in_between(
        "public-api/src/lib.rs",
        "pub const MINIMUM_NIGHTLY_RUST_VERSION: &str = \"",
        release_helper::version_info::TABLE[0].min_nightly_rust_version,
        "\";
// End-marker for scripts/release-helper/src/bin/update-version-info/main.rs",
    );
}

fn insert_file_contents_in_between(
    source_file_path: &str,
    existing_start_text: &str,
    new_text_in_middle: &str,
    existing_end_text: &str,
) {
    let contents = std::fs::read_to_string(source_file_path).unwrap();

    let start_index = contents.find(existing_start_text).unwrap();
    let end_index = contents.find(existing_end_text).unwrap();

    let start = contents[..start_index + existing_start_text.len()].to_string();
    let end = contents[end_index..].to_string();

    let new_contents = format!("{}{}{}", start, new_text_in_middle, end);
    std::fs::write(source_file_path, new_contents.clone()).unwrap();
}
