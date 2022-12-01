use std::path::Path;

use assert_cmd::assert::Assert;

pub trait AssertOrBless {
    fn stdout_or_bless(self, expected_file: impl AsRef<Path>) -> Assert;
}

impl AssertOrBless for Assert {
    /// Note that due to the way `expect_file!` macro expansion works, relative
    /// paths are relative to this file, i.e. relative to `./test-utils/src/`.
    fn stdout_or_bless(self, expected_file: impl AsRef<Path>) -> Assert {
        let stdout = String::from_utf8_lossy(&self.get_output().stdout);
        expect_test::expect_file![expected_file.as_ref()].assert_eq(&stdout);
        self
    }
}
