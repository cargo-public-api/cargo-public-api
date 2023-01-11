// deny in CI, only warn here
#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::disallowed_methods)]

use std::{ffi::OsStr, fs::read_to_string, path::PathBuf};

use public_api::MINIMUM_RUSTDOC_JSON_VERSION;

#[test]
fn newline_at_end_of_all_files() {
    // Change dir to repo root
    let repo_root = std::env::current_dir()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    std::env::set_current_dir(&repo_root).unwrap();

    // Get a list of all versioned files
    let repo_files: Vec<_> = String::from_utf8_lossy(
        &std::process::Command::new("git")
            .arg("ls-files")
            .output()
            .unwrap()
            .stdout,
    )
    .lines()
    .map(|p| PathBuf::from(&p.trim()))
    .collect();

    // Sanity check file count
    let file_count = repo_files.len();
    assert!(
        file_count > 100,
        "We expect more than {file_count} lines from `git ls-files` in `{repo_root:?}`",
    );

    // Setup the list of file extensions to check. Produced by:
    //
    //   git ls-files | grep -v -e \.rs -e \.toml -e \.md -e \.sh -e \.txt -e \.yml -e \.json
    let checked_extensions = ["json", "md", "rs", "sh", "toml", "txt", "yml"].map(OsStr::new);

    // Check each file
    let mut missing_newline = vec![];
    for file in &repo_files {
        if !checked_extensions.contains(&file.extension().unwrap_or_default()) {
            continue;
        }

        if !read_to_string(file)
            .map_err(|e| format!("Could not read {file:?}: {e:?}"))
            .unwrap()
            .ends_with('\n')
        {
            missing_newline.push(file);
        }
    }

    // Report files which are missing a newline at the end
    assert!(
        missing_newline.is_empty(),
        "These files are missing a newline at the end: {:?}",
        missing_newline,
    );
}

#[test]
fn installation_instructions_in_toplevel_readme() {
    let readme = include_str!("../../README.md");
    let expected_installation_instruction =
        format!("Ensure `{MINIMUM_RUSTDOC_JSON_VERSION}` or later");
    assert!(readme.contains(&expected_installation_instruction));
}
