//! See `docs/RELEASE.md` for information on how to use this.

#[derive(clap::Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(long, default_value_t = 6)]
    min_compatibility_matrix_rows: usize,

    #[arg(long, default_value_t = 6)]
    max_compatibility_matrix_months_back: i64,
}

fn main() {
    let args = <Args as clap::Parser>::parse();

    let current_min_nightly_rust_version =
        release_helper::version_info::TABLE[0].min_nightly_rust_version;

    // Update README.md
    insert_file_contents_in_between(
        "README.md",
        "# Compatibility Matrix

| Version          | Understands the rustdoc JSON output of  |
| ---------------- | --------------------------------------- |
",
        &release_helper::compatibility_matrix::render(
            release_helper::version_info::TABLE,
            Some(args.min_compatibility_matrix_rows),
            Some(args.max_compatibility_matrix_months_back),
        ),
        "| earlier versions | see [here]",
    );
    insert_file_contents_in_between(
        "README.md",
        "```sh
cargo +stable install cargo-public-api
```

Ensure **",
        current_min_nightly_rust_version,
        "** or later is installed (does not need to be the active toolchain)",
    );

    // Update public-api/src/lib.rs
    insert_file_contents_in_between(
        "public-api/src/lib.rs",
        "pub const MINIMUM_NIGHTLY_RUST_VERSION: &str = \"",
        current_min_nightly_rust_version,
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
    println!("Updating {source_file_path} by inserting:\n{new_text_in_middle}\n");
    let contents = std::fs::read_to_string(source_file_path).unwrap();

    let start_index = contents.find(existing_start_text).unwrap();
    let end_index = contents.find(existing_end_text).unwrap();

    let start = contents[..start_index + existing_start_text.len()].to_string();
    let end = contents[end_index..].to_string();

    let new_contents = format!("{start}{new_text_in_middle}{end}");
    std::fs::write(source_file_path, new_contents.clone()).unwrap();
}
