// deny in CI, only warn here
#![warn(clippy::all, clippy::pedantic)]

//! To update expected output it is in many cases sufficient to run
//! ```bash
//! ./scripts/bless-expected-output-for-tests.sh
//! ```

use std::ffi::OsStr;
use std::io::Write;
use std::{
    fs::OpenOptions,
    path::{Path, PathBuf},
};

use assert_cmd::assert::Assert;
use assert_cmd::Command;
use predicates::str::contains;

// rust-analyzer bug: https://github.com/rust-lang/rust-analyzer/issues/9173
#[path = "../../test-utils/src/lib.rs"]
mod test_utils;
use tempfile::tempdir;
use test_utils::assert_or_bless::AssertOrBless;
use test_utils::rustdoc_json_path_for_crate;

#[path = "../src/git_utils.rs"] // Say NO to copy-paste!
mod git_utils;

fn create_test_repo_with_dirty_git_tree() -> TestRepo {
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

    test_repo
}

// FIXME: This tests is ignored in CI due to some unknown issue with windows
#[test]
#[cfg_attr(all(target_family = "windows", in_ci), ignore)]
fn list_public_items() {
    let mut cmd = Command::cargo_bin("cargo-public-api").unwrap();

    // Other tests use --simplified. Here we use -s to make sure that also works
    cmd.arg("-s");

    cmd.args(["--manifest-path", "../public-api/Cargo.toml"]);
    cmd.assert()
        .stdout_or_bless("./tests/expected-output/public_api_list.txt")
        .success();
}

#[test]
fn list_public_items_with_lint_error() {
    let mut cmd = TestCmd::new_without_test_repo();
    cmd.args(["--manifest-path", "../test-apis/lint_error/Cargo.toml"]);
    cmd.assert()
        .stdout_or_bless("./tests/expected-output/lint_error_list.txt")
        .success();
}

// FIXME: This tests is ignored in CI due to some unknown issue with windows
#[test]
#[cfg_attr(all(target_family = "windows", in_ci), ignore)]
fn custom_toolchain() {
    let mut cmd = TestCmd::new();
    cmd.arg("--toolchain");
    cmd.arg("nightly");
    cmd.assert()
        .stdout_or_bless("./tests/expected-output/example_api-v0.3.0.txt")
        .success();
}

// FIXME: This tests is ignored in CI due to some unknown issue with windows
#[test]
#[cfg_attr(all(target_family = "windows", in_ci), ignore)]
fn list_public_items_explicit_manifest_path() {
    let test_repo = TestRepo::new();
    let mut test_repo_manifest = PathBuf::from(test_repo.path());
    test_repo_manifest.push("Cargo.toml");

    let mut cmd = cargo_public_api_cmd_simplified();
    cmd.arg("--manifest-path");
    cmd.arg(&test_repo_manifest);
    cmd.assert()
        .stdout_or_bless("./tests/expected-output/example_api-v0.3.0.txt")
        .success();
}

/// Make sure we can run the tool with a specified package from a virtual
/// manifest.
#[test]
fn list_public_items_via_package_spec() {
    let mut cmd = TestCmd::new_without_test_repo();
    cmd.current_dir("../test-apis/virtual-manifest");
    cmd.arg("--package");
    cmd.arg("specific-crate");
    cmd.assert()
        .stdout_or_bless("./tests/expected-output/specific-crate.txt")
        .success();
}

#[test]
fn target_arg() {
    // A bit of a hack but similar to how rustc bootstrap script does it:
    // https://github.com/rust-lang/rust/blob/1ce51982b8550c782ded466c1abff0d2b2e21c4e/src/bootstrap/bootstrap.py#L207-L219
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
    let mut cmd = TestCmd::new();
    cmd.arg("--target");
    cmd.arg(get_host_target_triple());
    cmd.assert()
        .stdout_or_bless("./tests/expected-output/test_repo_api_latest.txt")
        .success();
}

#[test]
fn virtual_manifest_error() {
    let mut cmd = TestCmd::new_without_test_repo();
    cmd.arg("--manifest-path");
    cmd.arg("../test-apis/virtual-manifest/Cargo.toml");
    cmd.assert()
        .stdout("")
        .stderr(contains(
            "Listing or diffing the public API of an entire workspace is not supported.",
        ))
        .failure();
}

#[test]
fn diff_public_items() {
    diff_public_items_impl("--diff-git-checkouts");
}

#[test]
fn diff_public_items_smart_diff() {
    diff_public_items_impl("--diff");
}

fn diff_public_items_impl(diff_arg: &str) {
    let mut cmd = TestCmd::new();
    let test_repo_path = cmd.test_repo_path().to_owned();
    let branch_before = git_utils::current_branch(&test_repo_path).unwrap().unwrap();
    cmd.arg(diff_arg);
    cmd.arg("v0.2.0");
    cmd.arg("v0.3.0");
    cmd.assert()
        .stdout_or_bless("./tests/expected-output/example_api_diff_v0.2.0_to_v0.3.0.txt")
        .success();
    let branch_after = git_utils::current_branch(&test_repo_path).unwrap().unwrap();

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
    git_utils::git_checkout("v0.1.1", path, true, false).unwrap();
    assert_eq!(None, git_utils::current_branch(path).unwrap());
    let before = git_utils::current_commit(path).unwrap();

    let mut cmd = cargo_public_api_cmd_simplified();
    cmd.current_dir(path);
    cmd.arg("--diff-git-checkouts");
    cmd.arg("v0.2.0");
    cmd.arg("v0.3.0");
    cmd.assert()
        .stdout_or_bless("./tests/expected-output/example_api_diff_v0.2.0_to_v0.3.0.txt")
        .success();

    let after = git_utils::current_commit(path).unwrap();
    assert_eq!(before, after);
}

/// Test that diffing fails if the git tree is dirty
#[test]
#[cfg_attr(target_family = "windows", ignore)]
fn diff_public_items_with_dirty_tree_fails() {
    let test_repo = create_test_repo_with_dirty_git_tree();

    // Make sure diffing does not destroy uncommitted data!
    let mut cmd = cargo_public_api_cmd_simplified();
    cmd.current_dir(&test_repo.path);
    cmd.arg("--diff-git-checkouts");
    cmd.arg("v0.2.0");
    cmd.arg("v0.3.0");
    cmd.assert()
        .stderr(contains(
            "Your local changes to the following files would be overwritten by checkout",
        ))
        .failure();
}

/// Test that diffing succeedes if the git tree is dirty and
/// `force-git-checkout` option is specified.
#[test]
#[cfg_attr(target_family = "windows", ignore)]
fn diff_public_items_with_dirty_tree_succeedes_with_force_option() {
    let test_repo = create_test_repo_with_dirty_git_tree();

    let mut cmd = cargo_public_api_cmd_simplified();
    cmd.current_dir(&test_repo.path);
    cmd.arg("--diff-git-checkouts");
    cmd.arg("v0.2.0");
    cmd.arg("v0.3.0");
    cmd.arg("--force-git-checkouts");
    cmd.assert()
        .stdout_or_bless("./tests/expected-output/example_api_diff_v0.2.0_to_v0.3.0.txt")
        .success();
}

/// Test that relative git references like HEAD and HEAD^ work
/// (even as the second diff target).
#[test]
fn diff_public_items_relative_refs() {
    let test_repo = TestRepo::new();

    // Pick a specific commit to serve as our HEAD
    let path = test_repo.path();
    git_utils::git_checkout("v0.3.0", path, true, false).unwrap();
    assert_eq!(None, git_utils::current_branch(path).unwrap());
    let before = git_utils::current_commit(path).unwrap();

    let mut cmd = cargo_public_api_cmd_simplified();
    cmd.current_dir(path);
    cmd.arg("--diff-git-checkouts");
    cmd.arg("HEAD^");
    cmd.arg("HEAD");
    cmd.assert()
        .stdout_or_bless("./tests/expected-output/example_api_diff_v0.2.0_to_v0.3.0.txt")
        .success();

    let after = git_utils::current_commit(path).unwrap();
    assert_eq!(before, after);
}

#[test]
fn deny_when_not_diffing() {
    test_deny_not_allowed(["--deny=all"]);
}

#[test]
fn deny_added_when_not_diffing() {
    test_deny_not_allowed(["--deny=added"]);
}

#[test]
fn deny_changed_when_not_diffing() {
    test_deny_not_allowed(["--deny=changed"]);
}

#[test]
fn deny_removed_when_not_diffing() {
    test_deny_not_allowed(["--deny=removed"]);
}

#[test]
fn deny_combination_when_not_diffing() {
    test_deny_not_allowed(["--deny=added", "--deny=changed", "--deny=removed"]);
}

fn test_deny_not_allowed(args: impl IntoIterator<Item = &'static str>) {
    let mut cmd = TestCmd::new();
    for arg in args {
        cmd.arg(arg);
    }
    cmd.assert()
        .stderr(contains("`--deny` can only be used when diffing"))
        .failure();
}

#[test]
fn deny_without_diff() {
    let mut cmd = TestCmd::new();
    cmd.arg("--diff-git-checkouts");
    cmd.arg("v0.1.0");
    cmd.arg("v0.1.1");
    cmd.arg("--deny=all");
    cmd.assert().success();
}

#[test]
fn deny_with_diff() {
    let mut cmd = TestCmd::new();
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
    let mut cmd = TestCmd::new();
    cmd.arg("--diff-git-checkouts");
    cmd.arg("v0.1.0");
    cmd.arg("v0.2.0");
    cmd.arg("--deny=added");
    cmd.assert()
        .stdout_or_bless("./tests/expected-output/example_api_diff_v0.1.0_to_v0.2.0.txt")
        .failure();
}

#[test]
fn deny_changed_with_diff() {
    let mut cmd = TestCmd::new();
    cmd.arg("--diff-git-checkouts");
    cmd.arg("v0.1.0");
    cmd.arg("v0.2.0");
    cmd.arg("--deny=changed");
    cmd.assert().failure();
}

#[test]
fn deny_removed_with_diff() {
    let mut cmd = TestCmd::new();
    cmd.arg("--diff-git-checkouts");
    cmd.arg("v0.2.0");
    cmd.arg("v0.3.0");
    cmd.arg("--deny=removed");
    cmd.assert()
        .stderr(contains(
            "The API diff is not allowed as per --deny: Removed items not allowed: [pub fn example_api::function(v1_param: example_api::Struct, v2_param: usize)]",
        ))
        .failure();
}

#[test]
fn deny_with_invalid_arg() {
    let mut cmd = TestCmd::new();
    cmd.arg("--diff-git-checkouts");
    cmd.arg("v0.2.0");
    cmd.arg("v0.3.0");
    cmd.arg("--deny=invalid");
    cmd.assert()
        .stderr(contains("'invalid' isn't a valid value"))
        .failure();
}

#[test]
fn diff_public_items_with_manifest_path() {
    let test_repo = TestRepo::new();
    let mut cmd = cargo_public_api_cmd_simplified();
    cmd.arg("--manifest-path");
    cmd.arg(format!(
        "{}/Cargo.toml",
        &test_repo.path.path().to_string_lossy()
    ));
    cmd.arg("--diff-git-checkouts");
    cmd.arg("v0.2.0");
    cmd.arg("v0.3.0");
    cmd.assert()
        .stdout_or_bless("./tests/expected-output/example_api_diff_v0.2.0_to_v0.3.0.txt")
        .success();
}

#[test]
fn diff_public_items_without_git_root() {
    let mut cmd = cargo_public_api_cmd_simplified();
    cmd.arg("--manifest-path");
    cmd.arg("/does/not/exist/Cargo.toml");
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
    let mut cmd = TestCmd::new();
    cmd.arg("--color=always");
    cmd.arg("--diff-git-checkouts");
    cmd.arg("v0.1.0");
    cmd.arg("v0.2.0");
    cmd.assert()
        .stdout_or_bless("./tests/expected-output/example_api_diff_v0.1.0_to_v0.2.0_colored.txt")
        .success();
}

// FIXME: This tests is ignored in CI due to some unknown issue with windows
#[test]
#[cfg_attr(all(target_family = "windows", in_ci), ignore)]
fn list_public_items_with_color() {
    let mut cmd = TestCmd::new();
    cmd.arg("--color=always");
    cmd.assert()
        .stdout_or_bless("./tests/expected-output/example_api_v0.3.0_colored.txt")
        .success();
}

#[test]
fn diff_public_items_from_files() {
    diff_public_items_from_files_impl("--diff-rustdoc-json");
}
#[test]
fn diff_public_items_from_files_smart_diff() {
    diff_public_items_from_files_impl("--diff");
}

fn diff_public_items_from_files_impl(diff_arg: &str) {
    // Create independent build dirs so all tests can run in parallel
    let build_dir = tempdir().unwrap();
    let build_dir2 = tempdir().unwrap();

    let old = rustdoc_json_path_for_crate("../test-apis/example_api-v0.1.0", &build_dir);
    let new = rustdoc_json_path_for_crate("../test-apis/example_api-v0.2.0", &build_dir2);
    let mut cmd = TestCmd::new_without_test_repo();
    cmd.arg(diff_arg);
    cmd.arg(old);
    cmd.arg(new);
    cmd.assert()
        .stdout_or_bless("./tests/expected-output/example_api_diff_v0.1.0_to_v0.2.0.txt")
        .success();
}

#[test]
fn diff_published() {
    diff_published_impl("--diff-published", "example_api@0.1.0");
}

#[test]
fn diff_published_smart_diff() {
    diff_published_impl("--diff", "example_api@0.1.0");
}

#[test]
fn diff_published_fallback() {
    diff_published_impl("--diff-published", "@0.1.0");
}

#[test]
fn diff_published_smart_diff_fallback() {
    diff_published_impl("--diff", "@0.1.0");
}

/// Diff against a published crate.
fn diff_published_impl(diff_arg: &str, spec: &str) {
    let mut cmd = TestCmd::new();
    cmd.arg(diff_arg);
    cmd.arg(spec);
    cmd.assert()
        .stdout_or_bless("./tests/expected-output/diff_published.txt")
        .success();
}

#[test]
fn diff_published_explicit_package() {
    let mut cmd = TestCmd::new();
    cmd.arg("-p");
    cmd.arg("example_api");
    cmd.arg("--diff-published");
    cmd.arg("@0.1.0");
    cmd.assert()
        .stdout_or_bless("./tests/expected-output/diff_published.txt")
        .success();
}

#[test]
fn list_public_items_from_json_file() {
    // Create independent build dir so all tests can run in parallel
    let build_dir = tempdir().unwrap();

    let json_file = rustdoc_json_path_for_crate("../test-apis/example_api-v0.3.0", &build_dir);
    let mut cmd = TestCmd::new_without_test_repo();
    cmd.arg("--rustdoc-json");
    cmd.arg(json_file);
    cmd.assert()
        .stdout_or_bless("./tests/expected-output/example_api-v0.3.0.txt")
        .success();
}

#[test]
fn diff_public_items_missing_one_arg() {
    let mut cmd = TestCmd::new();
    cmd.arg("--diff-git-checkouts");
    cmd.arg("v0.2.0");
    cmd.assert()
        .stderr(contains("requires 2 values, but 1 was provided"))
        .failure();
}

#[test]
fn verbose() {
    let mut cmd = TestCmd::new_without_test_repo();
    cmd.arg("--manifest-path");
    cmd.arg("../test-apis/lint_error/Cargo.toml");
    cmd.arg("--verbose");
    cmd.assert()
        .stdout(contains("Processing \""))
        .stdout(contains("rustdoc JSON missing referenced item"))
        .success();
}

#[test]
fn long_help() {
    let mut cmd = cargo_public_api_cmd_simplified();
    cmd.arg("--help");
    assert_presence_of_args_in_help(cmd);
}

#[test]
fn long_help_wraps() {
    let max_allowed_line_length = 105; // 100 with some margin

    let mut cmd = Command::cargo_bin("cargo-public-api").unwrap();
    cmd.arg("--help");

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    for line in stdout.lines() {
        assert!(
            line.len() <= max_allowed_line_length,
            "Found line larger than {max_allowed_line_length} chars! Text wrapping seems broken? Line: '{line}'"
        );
    }
}

#[test]
fn short_help() {
    let mut cmd = cargo_public_api_cmd_simplified();
    cmd.arg("-h");
    assert_presence_of_args_in_help(cmd);
}

fn assert_presence_of_args_in_help(mut cmd: Command) {
    cmd.assert()
        .stdout(contains("--simplified"))
        .stdout(contains("--manifest-path"))
        .stdout(contains("--diff-git-checkouts"))
        .success();
}

/// Helper to initialize a test crate git repo. Each test gets its own git repo
/// to use so that tests can run in parallel.
fn initialize_test_repo(dest: &Path) {
    test_utils::create_test_git_repo(dest, "../test-apis");
}

fn cargo_public_api_cmd_simplified() -> Command {
    let mut cmd = Command::cargo_bin("cargo-public-api").unwrap();

    // Simplify output by default since if we render all of our own items
    // properly, the risk is low that we will render Blanket Implementations and
    // Auto Trait Implementations items wrong. Instead we choose to have
    // dedicated tests for the rendering of such items.
    cmd.arg("--simplified");

    cmd
}

#[derive(Debug)]
struct F<'a> {
    all: bool,
    none: bool,
    features: &'a [&'a str],
}

impl<'a> F<'a> {
    fn none(mut self) -> Self {
        self.none = true;
        self
    }
    fn all(mut self) -> Self {
        self.all = true;
        self
    }
    fn new(features: &'a [&'a str]) -> Self {
        F {
            all: false,
            none: false,
            features,
        }
    }
}

impl std::fmt::Display for F<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.all {
            write!(f, "all")?;
        }
        if self.none {
            write!(f, "none")?;
        }
        for feat in self.features {
            write!(f, "{feat}")?;
        }
        Ok(())
    }
}

#[test]
fn features_all() {
    test_features(&F::new(&[]).all());
}

#[test]
fn features_none() {
    test_features(&F::new(&[]).none());
}

#[test]
fn features_a_b_c() {
    test_features(&F::new(&["feature_a", "feature_b", "feature_c"]).none());
}

#[test]
fn features_b() {
    test_features(&F::new(&["feature_b"]).none());
}

#[test]
fn features_b_c() {
    test_features(&F::new(&["feature_c"]).none()); // includes `feature_b`
}

fn test_features(features: &F) {
    let mut cmd = TestCmd::new_without_test_repo();
    cmd.current_dir("../test-apis/features");

    if features.none {
        cmd.arg("--no-default-features");
    }

    if features.all {
        cmd.arg("--all-features");
    }

    for feature in features.features {
        cmd.args(["--features", feature]);
    }

    cmd.assert()
        .stdout_or_bless(&format!(
            "./tests/expected-output/features-feat{features}.txt"
        ))
        .success();
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

/// To maximize parallelism of tests, each tests should have its own git repo
/// (if it needs a git repo for testing), as well as its own target (build) dir.
///
/// This helper represents a command to test and its (optional) git repo and
/// target dir.
///
/// It comes with a bunch of convenience methods ([`Self::arg()`], etc) to make
/// test code simpler.
struct TestCmd {
    /// The `cargo-public-api` command to run for the test.
    cmd: Command,

    /// A short-lived temporary git repo used for tests. Note that not all tests
    /// need a repo, so this is optional.
    test_repo: Option<TestRepo>,

    /// The `./target` directory for the test. Using one `./target` dir per test
    /// increases parallelism of tests. A tricker contention issue to solve is
    /// the fact that cargo also acquires locks on the global package cache:
    /// https://github.com/rust-lang/cargo/blob/ba607b23db8398723d659249d9abf5536bc322e5/src/cargo/util/config/mod.rs#L1733-L1738
    target_dir: tempfile::TempDir,
}

impl TestCmd {
    /// Create a new test command with its own test repo. The `current_dir` will
    /// be set to the dir of the repo.
    fn new() -> Self {
        Self::new_impl(true)
    }

    /// Create a new test command but do not create a test repo for it.
    fn new_without_test_repo() -> Self {
        Self::new_impl(false)
    }

    /// Create a new test command and set up its repo (if it needs one) and its
    /// own target dir.
    fn new_impl(with_test_repo: bool) -> Self {
        let mut test_cmd = Self {
            cmd: cargo_public_api_cmd_simplified(),
            test_repo: None,
            target_dir: tempfile::tempdir().unwrap(),
        };

        test_cmd
            .cmd
            .arg("--target-dir")
            .arg(test_cmd.target_dir.path());

        if with_test_repo {
            let new_test_repo = TestRepo::new();
            test_cmd.cmd.current_dir(&new_test_repo.path);
            test_cmd.test_repo = Some(new_test_repo);
        }

        test_cmd
    }

    pub fn test_repo_path(&self) -> &Path {
        self.test_repo
            .as_ref()
            .expect("Test repo must be created first!!")
            .path()
    }

    pub fn current_dir(&mut self, current_dir: impl AsRef<Path>) -> &mut Self {
        self.cmd.current_dir(current_dir);
        self
    }

    pub fn arg(&mut self, arg: impl AsRef<OsStr>) -> &mut Self {
        self.cmd.arg(arg);
        self
    }

    pub fn args(&mut self, args: impl IntoIterator<Item = impl AsRef<OsStr>>) -> &mut Self {
        self.cmd.args(args);
        self
    }

    pub fn assert(&mut self) -> Assert {
        self.cmd.assert()
    }
}
