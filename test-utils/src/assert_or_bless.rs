use std::{
    fs::read_to_string,
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::Context;
use assert_cmd::assert::Assert;
use pretty_assertions::assert_eq;
use tempfile::NamedTempFile;

pub trait AssertOrBless {
    fn stdout_or_bless(self, expected_file: impl AsRef<Path>) -> Assert;
}

impl AssertOrBless for Assert {
    fn stdout_or_bless(self, expected_file: impl AsRef<Path>) -> Assert {
        if std::env::var("BLESS").is_ok() {
            let stdout = &self.get_output().stdout;
            // Write to dest as an atomic operation so that we get the desired
            // outcome even if the presence of concurrently running tests that
            // bless to the same file
            write_to_file_atomically(expected_file, stdout);
            self
        } else {
            // Make into a string to show diff
            self.stdout(read_to_string_unwrap(expected_file))
        }
    }
}

pub fn assert_eq_or_bless(actual: &str, expected_path: impl AsRef<Path>) {
    if std::env::var("BLESS").is_ok() {
        // Write to dest as an atomic operation so that we get the desired
        // outcome even if the presence of concurrently running tests that
        // bless to the same file
        write_to_file_atomically(expected_path, actual.as_bytes());
    } else {
        // Make into a string to show diff
        assert_eq!(actual, read_to_string_unwrap(expected_path));
    }
}

pub fn write_to_file_atomically(path: impl AsRef<Path>, data: &[u8]) {
    let mut temp_file = NamedTempFile::new_in(&temp_dir()).unwrap();
    temp_file.write_all(data).unwrap();
    std::fs::rename(temp_file, path.as_ref())
        .with_context(|| format!("Failed to rename {:?}", path.as_ref()))
        .unwrap();
}

/// A version of [`std::fs::read_to_string`] that prints the path to the file in
/// case of errors.
fn read_to_string_unwrap(path: impl AsRef<Path>) -> String {
    read_to_string(&path)
        .with_context(|| format!("Failed to read {:?}", path.as_ref()))
        .unwrap()
}

fn temp_dir() -> PathBuf {
    // Prefer CARGO_TARGET_TMPDIR so that the temp file is on the same file
    // system as the destination file, to make `rename` atomic.
    match option_env!("CARGO_TARGET_TMPDIR") {
        Some(cargo_tmpdir) => PathBuf::from(cargo_tmpdir),
        None => std::env::temp_dir(),
    }
}
