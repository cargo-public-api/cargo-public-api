use std::{
    io::Write,
    path::{Path, PathBuf},
};

use assert_cmd::assert::Assert;
use tempfile::NamedTempFile;

pub trait AssertOrBless {
    fn stdout_or_bless(self, expected_file: impl AsRef<Path>) -> Assert;
}

impl AssertOrBless for Assert {
    fn stdout_or_bless(self, expected_file: impl AsRef<Path>) -> Assert {
        if std::env::var("BLESS").is_ok() {
            let mut temp_file = NamedTempFile::new_in(&temp_dir()).unwrap();
            temp_file.write_all(&self.get_output().stdout).unwrap();

            // Write to dest as an atomic operation so that we get the desired
            // outcome even if the presence of concurrently running tests that
            // bless to the same file
            std::fs::rename(temp_file, expected_file).unwrap();
            self
        } else {
            // Make into a string to show diff
            let expected =
                String::from_utf8(std::fs::read(&expected_file).unwrap_or_else(|_| {
                    panic!("couldn't read file: {:?}", expected_file.as_ref())
                }))
                .unwrap();
            self.stdout(expected)
        }
    }
}

fn temp_dir() -> PathBuf {
    // Prefer CARGO_TARGET_TMPDIR so that the temp file is on the same file
    // system as the destination file, to make `rename` atomic.
    match option_env!("CARGO_TARGET_TMPDIR") {
        Some(cargo_tmpdir) => PathBuf::from(cargo_tmpdir),
        None => std::env::temp_dir(),
    }
}
