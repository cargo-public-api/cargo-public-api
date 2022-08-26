// deny in CI, only warn here
#![warn(clippy::all, clippy::pedantic)]

//! To update expected output it is in many cases sufficient to run
//! ```bash
//! ./scripts/bless-expected-output-for-tests.sh
//! ```

use std::io::Write;
use std::{
    fs::OpenOptions,
    path::{Path, PathBuf},
};

use assert_cmd::Command;
use predicates::str::contains;

#[path = "../src/git_utils.rs"] // Say NO to copy-paste!
mod git_utils;

#[test]
fn list_public_items() {
    let cmd = Command::cargo_bin("cargo-public-api").unwrap();
    assert_presence_of_own_library_items(cmd);
}

#[test]
fn list_public_items_with_lint_error() {
    let mut cmd = Command::cargo_bin("cargo-public-api").unwrap();
    cmd.args(["--manifest-path", "../test-apis/lint_error/Cargo.toml"]);
    cmd.assert()
        .stdout(
            "pub mod lint_error\n\
            pub struct lint_error::MissingDocs\n\
            ",
        )
        .success();
}

#[test]
fn custom_toolchain() {
    let mut cmd = Command::cargo_bin("cargo-public-api").unwrap();
    cmd.args(["--toolchain", "+nightly"]);
    assert_presence_of_own_library_items(cmd);
}

#[test]
fn list_public_items_explicit_manifest_path() {
    let mut cmd = Command::cargo_bin("cargo-public-api").unwrap();
    cmd.arg("--manifest-path");
    cmd.arg(current_dir_and("Cargo.toml"));
    assert_presence_of_own_library_items(cmd);
}

#[test]
fn target_arg() {
    // Ugly hack to get the target triple of the host platform. If you know of a
    // better way, please change to it!
    fn get_host_target_triple() -> String {
        let mut cmd = std::process::Command::new("sh");
        cmd.arg("-c");
        cmd.arg("rustc -vV | sed -n 's/host: \\(.*\\)/\\1/gp'");
        let stdout = cmd.output().unwrap().stdout;
        String::from_utf8_lossy(&stdout)
            .to_string()
            .trim()
            .to_owned()
    }

    // Make sure to use a separate and temporary repo so that this test does not
    // accidentally pass due to files from other tests lying around
    let test_repo = TestRepo::new();
    let mut cmd = Command::cargo_bin("cargo-public-api").unwrap();
    cmd.current_dir(&test_repo.path);
    cmd.arg("--target");
    cmd.arg(get_host_target_triple());
    cmd.assert()
        .stdout(include_str!("./expected-output/test_repo_api_latest.txt"))
        .success();
}

#[test]
fn virtual_manifest_error() {
    let mut cmd = Command::cargo_bin("cargo-public-api").unwrap();
    cmd.arg("--manifest-path");
    cmd.arg(current_dir_and("tests/virtual-manifest/Cargo.toml"));
    cmd.assert()
        .stdout("")
        .stderr(contains(
            "Listing or diffing the public API of an entire workspace is not supported.",
        ))
        .failure();
}

#[test]
fn diff_public_items() {
    let test_repo = TestRepo::new();
    let branch_before = git_utils::current_branch(&test_repo.path).unwrap().unwrap();
    let mut cmd = Command::cargo_bin("cargo-public-api").unwrap();
    cmd.current_dir(&test_repo.path);
    cmd.arg("--color=never");
    cmd.arg("--diff-git-checkouts");
    cmd.arg("v0.2.0");
    cmd.arg("v0.3.0");
    cmd.assert()
        .stdout(include_str!(
            "./expected-output/example_api_diff_v0.2.0_to_v0.3.0.txt"
        ))
        .success();
    let branch_after = git_utils::current_branch(&test_repo.path).unwrap().unwrap();

    // Diffing does a git checkout of the commits to diff. Afterwards the
    // original branch shall be restored to minimize user disturbance.
    assert_eq!(branch_before, branch_after);
}

/// Test that the mechanism to restore the original git branch works even if
/// there is no current branch
#[test]
fn diff_public_items_detached_head() {
    let test_repo = TestRepo::new();

    // Detach HEAD
    let path = test_repo.path();
    git_utils::git_checkout("v0.1.1", path, true).unwrap();
    assert_eq!(None, git_utils::current_branch(path).unwrap());
    let before = git_utils::current_commit(path).unwrap();

    let mut cmd = Command::cargo_bin("cargo-public-api").unwrap();
    cmd.current_dir(path);
    cmd.arg("--color=never");
    cmd.arg("--diff-git-checkouts");
    cmd.arg("v0.2.0");
    cmd.arg("v0.3.0");
    cmd.assert()
        .stdout(include_str!(
            "./expected-output/example_api_diff_v0.2.0_to_v0.3.0.txt"
        ))
        .success();

    let after = git_utils::current_commit(path).unwrap();
    assert_eq!(before, after);
}

/// Test that diffing fails if the git tree is dirty
#[test]
fn diff_public_items_with_dirty_tree_fails() {
    let test_repo = TestRepo::new();

    // Make the tree dirty by appending a comment to src/lib.rs
    let mut lib_rs_path = test_repo.path.path().to_owned();
    lib_rs_path.push("src/lib.rs");

    let mut lib_rs = OpenOptions::new()
        .write(true)
        .append(true)
        .open(&lib_rs_path)
        .unwrap();

    writeln!(lib_rs, "// Make git tree dirty").unwrap();

    // Make sure diffing does not destroy uncommitted data!
    let mut cmd = Command::cargo_bin("cargo-public-api").unwrap();
    cmd.current_dir(&test_repo.path);
    cmd.arg("--color=never");
    cmd.arg("--diff-git-checkouts");
    cmd.arg("v0.2.0");
    cmd.arg("v0.3.0");
    cmd.assert()
        .stderr(contains("commit your changes or stash them"))
        .failure();
}

#[test]
fn deny_when_not_diffing() {
    let test_repo = TestRepo::new();

    let mut cmd = Command::cargo_bin("cargo-public-api").unwrap();
    cmd.current_dir(&test_repo.path);
    cmd.arg("--deny=all");
    cmd.assert()
        .stderr(contains("`--deny` can only be used when diffing"))
        .failure();
}

#[test]
fn deny_added_when_not_diffing() {
    let test_repo = TestRepo::new();

    let mut cmd = Command::cargo_bin("cargo-public-api").unwrap();
    cmd.current_dir(&test_repo.path);
    cmd.arg("--deny=added");
    cmd.assert()
        .stderr(contains("`--deny` can only be used when diffing"))
        .failure();
}

#[test]
fn deny_changed_when_not_diffing() {
    let test_repo = TestRepo::new();

    let mut cmd = Command::cargo_bin("cargo-public-api").unwrap();
    cmd.current_dir(&test_repo.path);
    cmd.arg("--deny=changed");
    cmd.assert()
        .stderr(contains("`--deny` can only be used when diffing"))
        .failure();
}

#[test]
fn deny_removed_when_not_diffing() {
    let test_repo = TestRepo::new();

    let mut cmd = Command::cargo_bin("cargo-public-api").unwrap();
    cmd.current_dir(&test_repo.path);
    cmd.arg("--deny=removed");
    cmd.assert()
        .stderr(contains("`--deny` can only be used when diffing"))
        .failure();
}

#[test]
fn deny_combination_when_not_diffing() {
    let test_repo = TestRepo::new();

    let mut cmd = Command::cargo_bin("cargo-public-api").unwrap();
    cmd.current_dir(&test_repo.path);
    cmd.arg("--deny=added");
    cmd.arg("--deny=changed");
    cmd.arg("--deny=removed");
    cmd.assert()
        .stderr(contains("`--deny` can only be used when diffing"))
        .failure();
}

#[test]
fn deny_without_diff() {
    let test_repo = TestRepo::new();
    let mut cmd = Command::cargo_bin("cargo-public-api").unwrap();
    cmd.current_dir(&test_repo.path);
    cmd.arg("--diff-git-checkouts");
    cmd.arg("v0.1.0");
    cmd.arg("v0.1.1");
    cmd.arg("--deny=all");
    cmd.assert().success();
}

#[test]
fn deny_with_diff() {
    let test_repo = TestRepo::new();
    let mut cmd = Command::cargo_bin("cargo-public-api").unwrap();
    cmd.current_dir(&test_repo.path);
    cmd.arg("--diff-git-checkouts");
    cmd.arg("v0.1.0");
    cmd.arg("v0.2.0");
    cmd.arg("--deny=all");
    cmd.assert()
        .stderr(contains("The API diff is not allowed as per --deny"))
        .failure();
}

#[test]
fn deny_added_with_diff() {
    let test_repo = TestRepo::new();

    let mut cmd = Command::cargo_bin("cargo-public-api").unwrap();
    cmd.current_dir(&test_repo.path);
    cmd.arg("--diff-git-checkouts");
    cmd.arg("v0.1.0");
    cmd.arg("v0.2.0");
    cmd.arg("--deny=added");
    cmd.assert()
        .stdout(
            "Removed items from the public API
=================================
(none)

Changed items in the public API
===============================
-pub fn example_api::function(v1_param: Struct)
+pub fn example_api::function(v1_param: Struct, v2_param: usize)
-pub struct example_api::Struct
+#[non_exhaustive] pub struct example_api::Struct

Added items to the public API
=============================
+pub struct example_api::StructV2
+pub struct field example_api::Struct::v2_field: usize
+pub struct field example_api::StructV2::field: usize

",
        )
        .failure();
}

#[test]
fn deny_changed_with_diff() {
    let test_repo = TestRepo::new();

    let mut cmd = Command::cargo_bin("cargo-public-api").unwrap();
    cmd.current_dir(&test_repo.path);
    cmd.arg("--diff-git-checkouts");
    cmd.arg("v0.1.0");
    cmd.arg("v0.2.0");
    cmd.arg("--deny=changed");
    cmd.assert().failure();
}

#[test]
fn deny_removed_with_diff() {
    let test_repo = TestRepo::new();

    let mut cmd = Command::cargo_bin("cargo-public-api").unwrap();
    cmd.current_dir(&test_repo.path);
    cmd.arg("--diff-git-checkouts");
    cmd.arg("v0.2.0");
    cmd.arg("v0.3.0");
    cmd.arg("--deny=removed");
    cmd.assert()
        .stderr(contains(
            "The API diff is not allowed as per --deny: Removed items not allowed: [pub fn example_api::function(v1_param: Struct, v2_param: usize)]",
        ))
        .failure();
}

#[test]
fn deny_with_invalid_arg() {
    let test_repo = TestRepo::new();
    let mut cmd = Command::cargo_bin("cargo-public-api").unwrap();
    cmd.current_dir(&test_repo.path);
    cmd.arg("--diff-git-checkouts");
    cmd.arg("v0.2.0");
    cmd.arg("v0.3.0");
    cmd.arg("--deny=invalid");
    cmd.assert()
        .stderr(contains("\"invalid\" isn't a valid value"))
        .failure();
}

#[test]
fn diff_public_items_with_manifest_path() {
    let test_repo = TestRepo::new();
    let mut cmd = Command::cargo_bin("cargo-public-api").unwrap();
    cmd.arg("--manifest-path");
    cmd.arg(format!(
        "{}/Cargo.toml",
        &test_repo.path.path().to_string_lossy()
    ));
    cmd.arg("--color=never");
    cmd.arg("--diff-git-checkouts");
    cmd.arg("v0.2.0");
    cmd.arg("v0.3.0");
    cmd.assert()
        .stdout(include_str!(
            "./expected-output/example_api_diff_v0.2.0_to_v0.3.0.txt"
        ))
        .success();
}

#[test]
fn diff_public_items_without_git_root() {
    let mut cmd = Command::cargo_bin("cargo-public-api").unwrap();
    cmd.arg("--manifest-path");
    cmd.arg("/does/not/exist/Cargo.toml");
    cmd.arg("--color=never");
    cmd.arg("--diff-git-checkouts");
    cmd.arg("v0.2.0");
    cmd.arg("v0.3.0");
    cmd.assert()
        .stderr(predicates::str::starts_with(
            "Error: No `.git` dir when starting from `",
        ))
        .failure();
}

#[test]
fn diff_public_items_with_color() {
    let test_repo = TestRepo::new();
    let mut cmd = Command::cargo_bin("cargo-public-api").unwrap();
    cmd.current_dir(&test_repo.path);
    cmd.arg("--color=always");
    cmd.arg("--diff-git-checkouts");
    cmd.arg("v0.1.0");
    cmd.arg("v0.2.0");
    cmd.assert()
        .stdout(include_str!(
            "./expected-output/example_api_diff_v0.1.0_to_v0.2.0_colored.txt"
        ))
        .success();
}

#[test]
fn list_public_items_with_color() {
    let mut cmd = Command::cargo_bin("cargo-public-api").unwrap();
    cmd.arg("--color=always");
    cmd.assert()
        .stdout(include_str!(
            "./expected-output/list_self_test_lib_items_colored.txt"
        ))
        .success();
}

#[test]
fn diff_public_items_markdown() {
    let test_repo = TestRepo::new();
    let mut cmd = Command::cargo_bin("cargo-public-api").unwrap();
    cmd.current_dir(&test_repo.path);
    cmd.arg("--output-format=markdown");
    cmd.arg("--diff-git-checkouts");
    cmd.arg("v0.1.0");
    cmd.arg("v0.2.0");
    cmd.assert()
        .stdout(
            r"## Removed items from the public API
(none)

## Changed items in the public API
* `pub fn example_api::function(v1_param: Struct)` changed to
  `pub fn example_api::function(v1_param: Struct, v2_param: usize)`
* `pub struct example_api::Struct` changed to
  `#[non_exhaustive] pub struct example_api::Struct`

## Added items to the public API
* `pub struct example_api::StructV2`
* `pub struct field example_api::Struct::v2_field: usize`
* `pub struct field example_api::StructV2::field: usize`

",
        )
        .success();
}

#[test]
fn diff_public_items_markdown_no_changes() {
    let test_repo = TestRepo::new();
    let mut cmd = Command::cargo_bin("cargo-public-api").unwrap();
    cmd.current_dir(&test_repo.path);
    cmd.arg("--output-format=markdown");
    cmd.arg("--diff-git-checkouts");
    cmd.arg("v0.1.0");
    cmd.arg("v0.1.1");
    cmd.assert()
        .stdout("(No changes to the public API)\n")
        .success();
}

#[test]
fn diff_public_items_from_files() {
    let mut cmd = Command::cargo_bin("cargo-public-api").unwrap();
    cmd.arg("--diff-rustdoc-json");
    cmd.arg("../public-api/tests/rustdoc-json/example_api-v0.1.0.json");
    cmd.arg("../public-api/tests/rustdoc-json/example_api-v0.2.0.json");
    cmd.assert()
        .stdout(
            "Removed items from the public API
=================================
(none)

Changed items in the public API
===============================
-pub fn example_api::function(v1_param: Struct)
+pub fn example_api::function(v1_param: Struct, v2_param: usize)
-pub struct example_api::Struct
+#[non_exhaustive] pub struct example_api::Struct

Added items to the public API
=============================
+pub struct example_api::StructV2
+pub struct field example_api::Struct::v2_field: usize
+pub struct field example_api::StructV2::field: usize

",
        )
        .success();
}

#[test]
fn diff_public_items_missing_one_arg() {
    let test_repo = TestRepo::new();
    let mut cmd = Command::cargo_bin("cargo-public-api").unwrap();
    cmd.current_dir(&test_repo.path);
    cmd.arg("--diff-git-checkouts");
    cmd.arg("v0.2.0");
    cmd.assert()
        .stderr(contains(
            "requires at least 2 values but only 1 was provided",
        ))
        .failure();
}

#[test]
fn list_public_items_markdown() {
    let mut cmd = Command::cargo_bin("cargo-public-api").unwrap();
    cmd.arg("--output-format=markdown");
    cmd.assert()
        .stdout(
            "## Public API\n\
             * `pub fn cargo_public_api::for_self_testing_purposes_please_ignore()`\n\
             * `pub mod cargo_public_api`\n\
             * `pub use cargo_public_api::public_api`\n\
             \n\
             ",
        )
        .success();
}

#[test]
fn verbose() {
    let mut cmd = Command::cargo_bin("cargo-public-api").unwrap();
    cmd.arg("--verbose");
    cmd.assert()
        .stdout(contains("Processing \""))
        .stdout(contains("rustdoc JSON missing referenced item"))
        .success();
}

#[test]
fn long_help() {
    let mut cmd = Command::cargo_bin("cargo-public-api").unwrap();
    cmd.arg("--help");
    assert_presence_of_args_in_help(cmd);
}

#[test]
fn short_help() {
    let mut cmd = Command::cargo_bin("cargo-public-api").unwrap();
    cmd.arg("-h");
    assert_presence_of_args_in_help(cmd);
}

fn assert_presence_of_own_library_items(mut cmd: Command) {
    cmd.assert()
        .stdout(
            "pub fn cargo_public_api::for_self_testing_purposes_please_ignore()\n\
             pub mod cargo_public_api\n\
             pub use cargo_public_api::public_api\n\
             ",
        )
        .success();
}

fn assert_presence_of_args_in_help(mut cmd: Command) {
    cmd.assert()
        .stdout(contains("--with-blanket-implementations"))
        .stdout(contains("--manifest-path"))
        .stdout(contains("--diff-git-checkouts"))
        .success();
}

/// Helper to get the absolute path to a given path, relative to the current
/// path
fn current_dir_and<P: AsRef<Path>>(path: P) -> PathBuf {
    let mut cur_dir = std::env::current_dir().unwrap();
    cur_dir.push(path);
    cur_dir
}

/// Helper to initialize a test crate git repo. Each test gets its own git repo
/// to use so that tests can run in parallel.
fn initialize_test_repo(dest: &Path) {
    let mut cmd = std::process::Command::new("../scripts/create-test-git-repo.sh");
    cmd.arg(dest);
    assert!(cmd.spawn().unwrap().wait().unwrap().success());
}

/// A git repository that lives during the duration of a test. Having each test
/// have its own git repository to test with makes tests runnable concurrently.
struct TestRepo {
    path: tempfile::TempDir,
}

impl TestRepo {
    fn new() -> Self {
        let tempdir = tempfile::tempdir().unwrap();
        initialize_test_repo(tempdir.path());

        Self { path: tempdir }
    }

    fn path(&self) -> &Path {
        self.path.path()
    }
}
