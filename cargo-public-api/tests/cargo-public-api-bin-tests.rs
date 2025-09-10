//! To update expected output it is in many cases sufficient to run
//! ```bash
//! ./scripts/cargo-test.sh --update-snapshots
//! ```

use std::env;
use std::env::consts::EXE_SUFFIX;
use std::ffi::OsStr;
use std::io::Write;
use std::{
    fs::OpenOptions,
    path::{Path, PathBuf},
};

use assert_cmd::Command;
use assert_cmd::assert::Assert;
use jiff::ToSpan;
use jiff::civil::Date;
use predicates::prelude::PredicateBooleanExt;
use predicates::str::contains;

use public_api::MINIMUM_NIGHTLY_RUST_VERSION;
use tempfile::tempdir;

// We don't want to libify cargo-public-api (at least not yet), but we don't
// want to copy-paste code or introduce extra dependencies either. So we do this
// little hack.
#[path = "../src/git_utils.rs"]
mod git_utils;

mod create_test_git_repo;

/// A toolchain that is old enough to produce compilation errors. In other
/// words, we never even try to parse any rustdoc JSON output, because we do not
/// get that far.
const COMPILATION_ERROR_TOOLCHAIN: &str = "nightly-2022-06-01";

#[test]
fn list_public_items() {
    let mut cmd = Command::cargo_bin("cargo-public-api").unwrap();

    // Other tests use --simplified. Here we use -s to make sure that also works
    cmd.arg("-s");

    cmd.args(["--manifest-path", "../public-api/Cargo.toml"]);
    cmd.assert()
        .stdout_with_insta("list_public_items")
        .success();
}

#[test]
fn list_public_items_omit_blanket_impls() {
    let mut cmd = TestCmd::as_subcommand_without_args().with_test_repo();
    cmd.arg("--omit");
    cmd.arg("blanket-impls");
    cmd.assert()
        .stdout_with_insta("omit-blanket-impls")
        .success();
}

#[test]
fn list_public_items_omit_auto_trait_impls_impls() {
    let mut cmd = TestCmd::as_subcommand_without_args().with_test_repo();
    cmd.arg("--omit");
    cmd.arg("auto-trait-impls");
    cmd.assert()
        .stdout_with_insta("omit-auto-trait-impls")
        .success();
}

#[test]
fn list_public_items_omit_auto_derived_impls() {
    let mut cmd = TestCmd::as_subcommand_without_args().with_test_repo();
    cmd.arg("--omit");
    cmd.arg("auto-derived-impls");
    cmd.assert()
        .stdout_with_insta("omit-auto-derived-impls")
        .success();
}

#[test]
fn list_public_items_omit_auto_derived_impls_with_double_s() {
    let mut cmd = TestCmd::as_subcommand_without_args().with_test_repo();
    cmd.arg("-ss"); // Note the double -s
    cmd.assert()
        .stdout_with_insta("omit-auto-derived-impls-with-double-s")
        .success();
}

#[test]
fn list_public_items_omit_auto_derived_impls_with_triple_s() {
    let mut cmd = TestCmd::as_subcommand_without_args().with_test_repo();
    cmd.arg("-sss"); // Note the triple -s
    cmd.assert()
        .stdout_with_insta("omit-auto-derived-impls-with-triple-s")
        .success();
}

#[test]
fn list_public_items_with_lint_error() {
    let mut cmd = TestCmd::new().with_separate_target_dir();
    cmd.args(["--manifest-path", "../test-apis/lint_error/Cargo.toml"]);
    cmd.assert().stdout_with_insta("lint_error_list").success();
}

/// Ensure we can handle when
///
/// ```
/// [lib]
/// name = "foo"
/// ```
///
/// has a different name than
///
/// ```
/// [package]
/// name = "bar"
/// ```
///
/// in `Cargo.toml`.
#[test]
fn list_public_items_with_other_lib_name() {
    let mut cmd = TestCmd::new().with_separate_target_dir();
    cmd.args(["--manifest-path", "../test-apis/other-lib-name/Cargo.toml"]);
    cmd.assert().stdout_with_insta("other_lib_name").success();
}

#[test]
fn list_public_items_with_no_lib() {
    let mut cmd = TestCmd::new().with_separate_target_dir();
    cmd.args(["--manifest-path", "../test-apis/no-lib/Cargo.toml"]);
    cmd.assert()
        .stderr(contains("error")) // Be aware of ANSI color escape codes!
        .stderr(contains("no library targets found in package `no-lib`"))
        .failure();
}

/// Test that `cargo-public-api` can be renamed to `cargo-public-api-v0.13.0`
/// and still be invoked as `cargo public-api-v0.13.0` i.e. as a cargo
/// subcommand.
#[test]
fn renamed_binary_works_as_subcommand() {
    assert_cargo_public_api_in_path();

    // Create a `cargo-public-api-v0.13.0` binary by copying `cargo-public-api`.
    let bin_dir = bin_dir();
    let regular_bin = bin_dir.join(format!("cargo-public-api{EXE_SUFFIX}"));
    let renamed_bin = RmOnDrop(bin_dir.join(format!("cargo-public-api-v0.13.0{EXE_SUFFIX}")));
    std::fs::copy(regular_bin, &renamed_bin.0).unwrap();

    // Make sure the renamed binary can be invoked as a subcommand.
    let mut cmd = std::process::Command::new("cargo");
    cmd.arg("public-api-v0.13.0");
    cmd.arg("-h");
    Command::from_std(cmd)
        .assert()
        .stderr(contains(""))
        .stdout_with_insta("short-help")
        .success();
}

#[test]
fn debug_logging() {
    // From `info!("Running {cmd:?}")` at rustdoc-json/src/builder.rs.
    let indication_1 = "Running ";

    // From `#[instrument(...)]` on `fn rustdoc_json_path_for_manifest_path()`.
    let indication_2 = "rustdoc_json_path_for_manifest_path";

    let mut cmd = TestCmd::new().with_test_repo();
    cmd.cmd().env("RUST_LOG", "debug");
    cmd.assert()
        .stderr(
            predicates::str::contains(indication_1).and(predicates::str::contains(indication_2)),
        )
        .stdout_with_insta("test_repo_api_latest")
        .success();
}

/// This allows us to test two things, namely that
/// * [`MINIMUM_NIGHTLY_RUST_VERSION`] is not set too high
/// * `cargo pubic-api` suggests the nightly toolchain might be too old when a
///   too old nightly toolchain is used
///
/// Note: If there is no rustdoc JSON incompatibilities in the previous version,
/// this test will fail. In that case, feel to add an `#[ignore]` in the
/// meantime.
#[test]
#[ignore = "this test assumes rustdoc JSON incomaptibilities in the previous version but that is not the case right now"]
fn one_day_before_minimum_nightly_rust_version() {
    test_unusable_toolchain(
        TestCmd::with_proxy_toolchain(&get_toolchain_one_day_before_minimal_toolchain())
            .with_separate_target_dir(),
        &format!(
            "This version of `cargo public-api` requires at least:

    {MINIMUM_NIGHTLY_RUST_VERSION}

"
        ),
    );
}

/// Test that we can use a custom toolchain by using a toolchain that should
/// result in a compilation error. If we get a compilation error, we know that
/// the custom toolchain is being used.
///
/// This test uses the `--toolchain` option.
#[test]
fn compilation_error_toolchain() {
    test_unusable_toolchain(
        TestCmd::with_proxy_toolchain(COMPILATION_ERROR_TOOLCHAIN).with_separate_target_dir(),
        "generic associated types are unstable",
    );
}

/// Test that we can specify a custom toolchain via the `+toolchain` mechanism.
/// This also differs slightly from `compilation_error_toolchain()` by checking
/// for a generic error message rather than a specific one.
#[test]
fn custom_toolchain_via_proxy() {
    test_unusable_toolchain(
        TestCmd::with_proxy_toolchain(COMPILATION_ERROR_TOOLCHAIN).with_separate_target_dir(),
        "Failed to build rustdoc JSON",
    );
}

/// Test to make sure a custom toolchain can be used. Run the test with an
/// unusable toolchain. If the command fails, we assume that the unusable
/// toolchain was used, i.e. the test pass.
///
/// For more info on the rustup proxy mechanism, see
/// <https://rust-lang.github.io/rustup/concepts/index.html#how-rustup-works>.
fn test_unusable_toolchain(mut cmd: TestCmd, expected_stderr: &str) {
    // Test against comprehensive_api, because we want any rustdoc JSON format
    // incompatibilities to be detected
    cmd.args([
        "--manifest-path",
        "../test-apis/comprehensive_api/Cargo.toml",
    ]);

    // The test uses a too old nightly toolchain, which should make the tool
    // fail if it's used. If it fails, we assume the custom toolchain is being
    // used.
    cmd.assert().stderr(contains(expected_stderr)).failure();
}

#[test]
fn list_public_items_explicit_manifest_path() {
    let test_repo = TestRepo::new();
    let mut test_repo_manifest = PathBuf::from(test_repo.path());
    test_repo_manifest.push("Cargo.toml");

    let mut cmd = TestCmd::new();
    cmd.arg("--manifest-path");
    cmd.arg(&test_repo_manifest);
    cmd.assert()
        .stdout_with_insta("example_api-v0.3.0")
        .success();
}

/// Make sure we can run the tool with a specified package from a virtual
/// manifest.
#[test]
fn list_public_items_via_package_spec() {
    let mut cmd = TestCmd::new().with_separate_target_dir();
    cmd.current_dir("../test-apis/virtual-manifest");
    cmd.arg("--package");
    cmd.arg("specific-crate");
    cmd.assert().stdout_with_insta("specific-crate").success();
}

#[test]
fn target_arg() {
    // A bit of a hack but similar to how rustc bootstrap script does it:
    // https://github.com/rust-lang/rust/blob/1ce51982b8550c782ded466c1abff0d2b2e21c4e/src/bootstrap/bootstrap.py#L207-L219
    fn get_host_target_triple() -> String {
        let mut cmd = std::process::Command::new("rustc");
        cmd.arg("-vV");
        let output = cmd.output().unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);
        stdout
            .lines()
            .find_map(|line| line.strip_prefix("host: "))
            .unwrap()
            .to_owned()
    }

    // Make sure to use a separate and temporary repo so that this test does not
    // accidentally pass due to files from other tests lying around
    let mut cmd = TestCmd::new().with_test_repo();
    cmd.arg("--target");
    cmd.arg(get_host_target_triple());
    cmd.assert()
        .stdout_with_insta("test_repo_api_latest")
        .success();
}

#[test]
fn virtual_manifest_error() {
    let mut cmd = TestCmd::new().with_separate_target_dir();
    cmd.arg("--manifest-path");
    cmd.arg("../test-apis/virtual-manifest/Cargo.toml");
    cmd.assert()
        .stdout("")
        .stderr(contains(
            "Listing or diffing the public API of an entire workspace is not supported.",
        ))
        .failure();
}

/// Make sure we can run the tool on the current directory as a cargo
/// sub-command without any args
#[test]
fn subcommand_invocation() {
    let mut cmd = TestCmd::as_subcommand_without_args()
        .without_cargo_colors()
        .with_test_repo();
    cmd.assert()
        .stdout_with_insta("test_repo_api_latest_not_simplified")
        // Sanity check that rustdoc JSON build progress is shown to users, i.e.
        // that we do not swallow stderr from the cargo rustdoc JSON building
        // subprocess
        .stderr(contains("Documenting example_api"))
        .success();
}

/// Make sure we can run the tool on an external directory as a cargo sub-command
#[test]
fn subcommand_invocation_external_manifest() {
    let mut cmd = TestCmd::as_subcommand().with_separate_target_dir();
    cmd.args([
        "--manifest-path",
        "../test-apis/example_api-v0.3.0/Cargo.toml",
    ]);
    cmd.assert()
        .stdout_with_insta("example_api-v0.3.0")
        .success();
}

/// Make sure cargo subcommand args filtering of 'public-api' is not too
/// aggressive This tests `cargo public-api -p public-api`, and we want to
/// remove only the first `public-api` when we filter args (see `fn get_args()`
/// in `cargo-public-api/src/main.rs`)
#[test]
fn subcommand_invocation_public_api_arg() {
    // Don't use a separate target dir, because `public-api` is slow to build
    // from scratch. This is the only test that uses the root target dir, so
    // shared-resource contention on the .cargo-lock should not be an issue.
    let mut cmd = TestCmd::as_subcommand();
    cmd.current_dir(".."); // Enter git repo root so -p starts working
    cmd.args(["-p", "public-api"]);
    cmd.assert()
        .stdout_with_insta("subcommand_invocation_public_api_arg")
        .success();
}

/// The oldest nightly toolchain that we support. Sometimes the minimum toolchain
/// required for tests is not the same as required for users, so allow tests to
/// use a different toolchain if needed
fn get_minimum_toolchain() -> String {
    std::fs::read_to_string("../cargo-public-api/MINIMUM_NIGHTLY_RUST_VERSION_FOR_TESTS")
        .map(|s| {
            let min_for_tests = s.trim();
            if date_from_nightly_version(min_for_tests) < date_from_nightly_version(MINIMUM_NIGHTLY_RUST_VERSION) {
                core::panic!("You forgot to run\n\n    rm cargo-public-api/MINIMUM_NIGHTLY_RUST_VERSION_FOR_TESTS\n\nso please do it now.");
            }
            min_for_tests.to_owned()
        })
        .ok()
        .unwrap_or_else(|| MINIMUM_NIGHTLY_RUST_VERSION.to_owned())
}

#[test]
fn minimal_toolchain_works() {
    let mut cmd =
        TestCmd::with_proxy_toolchain(&get_minimum_toolchain()).with_separate_target_dir();

    // Test against comprehensive_api, because we want any rustdoc JSON format
    // incompatibilities to be detected
    cmd.args([
        "--manifest-path",
        "../test-apis/comprehensive_api/Cargo.toml",
    ]);

    cmd.assert()
        .stdout_with_insta("comprehensive_api")
        .success();
}

#[test]
fn warn_when_using_beta() {
    let mut cmd = TestCmd::with_proxy_toolchain("beta").with_separate_target_dir();

    // Test against comprehensive_api, because we want any rustdoc JSON format
    // incompatibilities to be detected
    cmd.args([
        "--manifest-path",
        "../test-apis/comprehensive_api/Cargo.toml",
    ]);

    cmd.assert()
        .stderr(contains("Warning: using the `beta"))
        .stderr(contains(
            "` toolchain for gathering the public api is not possible",
        ))
        .stdout_with_insta("comprehensive_api")
        .success();
}

#[test]
fn diff_public_items() {
    let mut cmd = TestCmd::new().with_test_repo();
    let test_repo_path = cmd.test_repo_path().to_owned();
    let branch_before = git_utils::current_branch(&test_repo_path).unwrap().unwrap();
    cmd.arg("diff");
    cmd.arg("v0.2.0..v0.3.0");
    cmd.assert()
        .stdout_with_insta("example_api_diff_v0.2.0_to_v0.3.0")
        .success();
    let branch_after = git_utils::current_branch(&test_repo_path).unwrap().unwrap();

    // Diffing does a git checkout of the commits to diff. Afterwards the
    // original branch shall be restored to minimize user disturbance.
    assert_eq!(branch_before, branch_after);
}

#[test]
fn diff_public_items_with_subcommand() {
    let mut cmd = TestCmd::new().with_test_repo();
    cmd.arg("diff");
    cmd.arg("v0.2.0..v0.3.0");
    cmd.assert()
        .stdout_with_insta("example_api_diff_v0.2.0_to_v0.3.0")
        .success();
}

#[test]
fn diff_with_invalid_published_crate_version() {
    let mut cmd = TestCmd::new().with_test_repo();
    cmd.arg("diff");
    cmd.arg("foo");
    cmd.assert()
        .stderr("Error: Invalid published crate version syntax: foo\n")
        .failure();
}

#[test]
fn diff_with_crate_not_published() {
    let tempdir = tempfile::tempdir().unwrap();

    let write_file = |name: &str, contents: &str| -> std::io::Result<PathBuf> {
        let mut path = tempdir.path().to_owned();
        path.push(name);
        std::fs::write(&path, contents)?;
        Ok(path)
    };

    let manifest = toml::toml! {
        [package]
        name = "this-create-has-not-been-published-and-never-will"
        version = "0.1.0"
        [lib]
        path = "lib.rs"
    };

    write_file("Cargo.toml", &manifest.to_string()).unwrap();
    write_file("lib.rs", "// empty lib").unwrap();

    let mut cmd = TestCmd::new();
    cmd.current_dir(tempdir.path());
    cmd.arg("diff");
    cmd.assert()
        .stderr(contains(
            "Error: Could not find crate `this-create-has-not-been-published-and-never-will`",
        ))
        .failure();
}

#[test]
fn diff_with_invalid_published_crate_version_number() {
    let mut cmd = TestCmd::new().with_test_repo();
    cmd.arg("diff");
    cmd.arg("9999.9999.9999");
    cmd.assert()
        .stderr(predicates::str::starts_with(
            "Error: Could not find version `9999.9999.9999` of crate `example_api`",
        ))
        .failure();
}

#[test]
fn diff_with_invalid_manifest_path() {
    let mut cmd = TestCmd::new().with_test_repo();
    cmd.arg("--manifest-path=/does/not/exists/Cargo.toml");
    cmd.arg("diff");
    cmd.arg("0.1.0");
    cmd.assert()
        .stderr(contains("You must specify a package"))
        .failure();
}

#[test]
fn diff_with_invalid_git_refs() {
    let mut cmd = TestCmd::new().with_test_repo();
    cmd.arg("diff");
    cmd.arg("foo..bar");
    cmd.assert()
        .stderr(contains("Error: fatal: ambiguous argument 'foo': unknown revision or path not in the working tree."))
        .failure();
}

#[test]
fn diff_with_invalid_git_refs_three_dots() {
    let mut cmd = TestCmd::new().with_test_repo();
    cmd.arg("diff");
    cmd.arg("foo...bar");
    cmd.assert()
        .stderr("Error: Invalid git diff syntax: foo...bar. Use: rev1..rev2\n")
        .failure();
}

#[test]
fn diff_with_invalid_git_refs_four_dots() {
    let mut cmd = TestCmd::new().with_test_repo();
    cmd.arg("diff");
    cmd.arg("foo....bar");
    cmd.assert()
        .stderr("Error: Invalid git diff syntax: foo....bar. Use: rev1..rev2\n")
        .failure();
}

#[test]
fn diff_with_three_args() {
    let mut cmd = TestCmd::new().with_test_repo();
    cmd.arg("diff");
    cmd.arg("v0.1.0");
    cmd.arg("v0.2.0");
    cmd.arg("v0.3.0");
    cmd.assert()
        .stderr("Error: Expected 1 or 2 arguments, but got 3\n")
        .failure();
}

#[test]
fn diff_with_dots_two_times() {
    let mut cmd = TestCmd::new().with_test_repo();
    cmd.arg("diff");
    cmd.arg("v0.1.0..v0.2.0");
    cmd.arg("v0.2.0..v0.3.0");
    cmd.assert()
        .stderr("Error: Use `ref1..ref2` syntax to diff git commits\n")
        .failure();
}

/// Test that the mechanism to restore the original git branch works even if
/// there is no current branch
#[test]
fn diff_public_items_detached_head() {
    let test_repo = TestRepo::new();

    // Detach HEAD
    let path = test_repo.path();
    git_utils::git_checkout(path, "v0.1.1", true, false).unwrap();
    assert_eq!(None, git_utils::current_branch(path).unwrap());
    let before = git_utils::current_commit(path).unwrap();

    let mut cmd = TestCmd::new();
    cmd.current_dir(path);
    cmd.arg("diff");
    cmd.arg("v0.2.0..v0.3.0");
    cmd.assert()
        .stdout_with_insta("example_api_diff_v0.2.0_to_v0.3.0")
        .success();

    let after = git_utils::current_commit(path).unwrap();
    assert_eq!(before, after);
}

/// Test that diffing fails if the git tree is dirty
#[test]
fn diff_public_items_with_dirty_tree_fails() {
    let test_repo = create_test_repo_with_dirty_git_tree();

    // Make sure diffing does not destroy uncommitted data!
    let mut cmd = TestCmd::new();
    cmd.current_dir(test_repo.path());
    cmd.arg("diff");
    cmd.arg("v0.2.0..v0.3.0");
    cmd.assert()
        .stderr(contains(
            "Your local changes to the following files would be overwritten by checkout",
        ))
        .failure();
}

/// Test that diffing succeeds if the git tree is dirty and
/// `force-git-checkout` option is specified.
#[test]
fn diff_public_items_with_dirty_tree_succeeds_with_force_option() {
    let test_repo = create_test_repo_with_dirty_git_tree();

    let mut cmd = TestCmd::new();
    cmd.current_dir(test_repo.path());
    cmd.arg("diff");
    cmd.arg("v0.2.0..v0.3.0");
    cmd.arg("--force");
    cmd.assert()
        .stdout_with_insta("example_api_diff_v0.2.0_to_v0.3.0")
        .success();
}

/// Test that relative git references like HEAD and HEAD^ work
/// (even as the second diff target).
#[test]
fn diff_public_items_relative_refs() {
    let test_repo = TestRepo::new();

    // Pick a specific commit to serve as our HEAD
    let path = test_repo.path();
    git_utils::git_checkout(path, "v0.3.0", true, false).unwrap();
    assert_eq!(None, git_utils::current_branch(path).unwrap());
    let before = git_utils::current_commit(path).unwrap();

    let mut cmd = TestCmd::new();
    cmd.current_dir(path);
    cmd.arg("diff");
    cmd.arg("HEAD^..HEAD");
    cmd.assert()
        .stdout_with_insta("example_api_diff_v0.2.0_to_v0.3.0")
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
    let mut cmd = TestCmd::new().with_test_repo();
    for arg in args {
        cmd.arg(arg);
    }
    // The exact phrasing of the error message on stderr varies with clap
    // version, and it is annoying to keep it up to date. So just assert on
    // .failure() and not on the specifics of the  error message.
    cmd.assert().failure();
}

#[test]
fn deny_without_diff() {
    let mut cmd = TestCmd::new().with_test_repo();
    cmd.arg("diff");
    cmd.arg("v0.1.0..v0.1.1");
    cmd.arg("--deny=all");
    cmd.assert().success();
}

#[test]
fn deny_with_diff() {
    let mut cmd = TestCmd::new().with_test_repo();
    cmd.arg("diff");
    cmd.arg("v0.1.0..v0.2.0");
    cmd.arg("--deny=all");
    cmd.assert()
        .stderr(contains("The API diff is not allowed as per --deny"))
        .failure();
}

#[test]
fn deny_with_diff_with_subcommand() {
    let mut cmd = TestCmd::new().with_test_repo();
    cmd.arg("diff");
    cmd.arg("v0.1.0..v0.2.0");
    cmd.arg("--deny=all");
    cmd.assert()
        .stderr(contains("The API diff is not allowed as per --deny"))
        .failure();
}

#[test]
fn deny_added_with_diff() {
    let mut cmd = TestCmd::new().with_test_repo();
    cmd.arg("diff");
    cmd.arg("v0.1.0..v0.2.0");
    cmd.arg("--deny=added");
    cmd.assert()
        .stdout_with_insta("example_api_diff_v0.1.0_to_v0.2.0")
        .failure();
}

#[test]
fn deny_changed_with_diff() {
    let mut cmd = TestCmd::new().with_test_repo();
    cmd.arg("diff");
    cmd.arg("v0.1.0..v0.2.0");
    cmd.arg("--deny=changed");
    cmd.assert().failure();
}

#[test]
fn deny_removed_with_diff() {
    let mut cmd = TestCmd::new().with_test_repo();
    cmd.arg("diff");
    cmd.arg("v0.2.0..v0.3.0");
    cmd.arg("--deny=removed");
    cmd.assert()
        .stderr(contains(
            "The API diff is not allowed as per --deny: Removed items not allowed: [pub fn example_api::function(v1_param: example_api::Struct, v2_param: usize)]",
        ))
        .failure();
}

#[test]
fn deny_with_invalid_arg() {
    let mut cmd = TestCmd::new().with_test_repo();
    cmd.arg("diff");
    cmd.arg("v0.2.0..v0.3.0");
    cmd.arg("--deny=invalid");
    cmd.assert().failure();
}

#[test]
fn diff_public_items_with_manifest_path() {
    let test_repo = TestRepo::new();
    let mut cmd = TestCmd::new();
    cmd.arg("--manifest-path");
    cmd.arg(format!(
        "{}/Cargo.toml",
        &test_repo.path().to_string_lossy()
    ));
    cmd.arg("diff");
    cmd.arg("v0.2.0..v0.3.0");
    cmd.assert()
        .stdout_with_insta("example_api_diff_v0.2.0_to_v0.3.0")
        .success();
}

#[test]
fn diff_public_items_without_git_root() {
    let mut cmd = TestCmd::new();
    cmd.arg("--manifest-path");
    cmd.arg("/does/not/exist/Cargo.toml");
    cmd.arg("diff");
    cmd.arg("v0.2.0..v0.3.0");
    cmd.assert().failure();
}

#[test]
fn diff_public_items_with_color() {
    let mut cmd = TestCmd::new().with_test_repo();
    cmd.arg("--color=always");
    cmd.arg("diff");
    cmd.arg("v0.1.0..v0.2.0");
    cmd.assert()
        .stdout_with_insta("example_api_diff_v0.1.0_to_v0.2.0_colored")
        .success();
}

#[test]
fn diff_public_items_with_color_arg_after_diff_subcommand() {
    let mut cmd = TestCmd::new().with_test_repo();
    cmd.arg("diff");
    cmd.arg("v0.1.0..v0.2.0");
    cmd.arg("--color=always");
    cmd.assert()
        .stdout_with_insta("example_api_diff_v0.1.0_to_v0.2.0_colored")
        .success();
}

#[test]
fn list_public_items_with_color() {
    list_public_items_with_color_impl("--color=always");
}

#[test]
fn list_public_items_with_color_implicitly() {
    list_public_items_with_color_impl("--color"); // Same as `--color=always`
}

fn list_public_items_with_color_impl(color_arg: &str) {
    let mut cmd = TestCmd::new().with_test_repo();
    cmd.arg(color_arg);
    cmd.assert()
        .stdout_with_insta("example_api_v0.3.0_colored")
        .success();
}

#[test]
fn diff_public_items_from_files_with_subcommand() {
    // Create independent build dirs so all tests can run in parallel
    let build_dir = tempdir().unwrap();
    let build_dir2 = tempdir().unwrap();

    let old = rustdoc_json_path_for_crate("../test-apis/example_api-v0.1.0", &build_dir);
    let new = rustdoc_json_path_for_crate("../test-apis/example_api-v0.2.0", &build_dir2);
    let mut cmd = TestCmd::new().with_separate_target_dir();
    cmd.arg("diff");
    cmd.arg(old);
    cmd.arg(new);
    cmd.assert()
        .stdout_with_insta("example_api_diff_v0.1.0_to_v0.2.0")
        .success();
}

#[test]
fn document_private_items() {
    // Create independent build dir so all tests can run in parallel
    let build_dir = tempdir().unwrap();

    let json = rustdoc_json_builder_for_crate("../test-apis/example_api-v0.3.0", &build_dir)
        .document_private_items(true)
        .build()
        .unwrap();
    let mut cmd = TestCmd::new().with_separate_target_dir();
    cmd.arg("--rustdoc-json");
    cmd.arg(json);
    cmd.assert()
        .stdout_with_insta("example_api-v0.3.0_document-private-items")
        .success();
}

#[test]
fn cap_lints_allow_by_default_when_diffing() {
    let mut cmd = TestCmd::new().with_test_repo_variant(TestRepoVariant::LintError);
    cmd.arg("diff");
    cmd.arg("v0.1.0..v0.1.1");

    // If `missing_docs` is printed, it must mean that the lint was not capped.
    // So require it to be absent in the output, since by default we do not want
    // to show lint errors when diffing. Because the user typically can't do
    // anything about it.
    cmd.assert()
        .stderr(contains("missing_docs").not())
        .success();
}

#[test]
fn diff_against_published_version() {
    let mut cmd = TestCmd::new().with_test_repo();
    cmd.arg("diff");
    cmd.arg("0.1.0");
    cmd.assert().stdout_with_insta("diff_published").success();
}

#[test]
fn diff_against_published_version_with_lib_name_different_from_package_name() {
    let mut cmd = TestCmd::new();
    cmd.arg("--manifest-path");
    cmd.arg("../test-apis/other-lib-name/Cargo.toml");
    cmd.arg("diff");
    cmd.arg("0.1.0");
    cmd.assert()
        .stdout_with_insta("other-lib-name-diff")
        .success();
}

/// Tests that we can diff between two published versions of an arbitrary crate
/// that does not need to be in the current workspace.
#[test]
fn diff_between_two_published_versions() {
    let mut cmd = TestCmd::new(); // NOTE: No `.with_test_repo()` !;
    cmd.arg("-p");
    cmd.arg("example_api");
    cmd.arg("diff");
    cmd.arg("0.1.0");
    cmd.arg("0.2.0");
    cmd.assert()
        .stdout_with_insta("example_api_diff_v0.1.0_to_v0.2.0")
        .success();
}

/// Test that `cargo public-api diff latest` works
#[test]
fn diff_against_latest_published_version() {
    diff_against_latest_published_version_impl(
        Some("latest"),
        "Resolved `diff latest` to `diff 0.3.0`",
    );
}

/// Test that `cargo public-api diff` resolves to `cargo public-api diff latest`
#[test]
fn diff_implicitly_against_latest_published_version() {
    diff_against_latest_published_version_impl(None, "Resolved `diff` to `diff 0.3.0`");
}

fn diff_against_latest_published_version_impl(arg: Option<&str>, expected_stderr: &str) {
    // Create a test repo. It already is at the latest version
    let test_repo = TestRepo::new();

    // Add a new item to the public API
    append_to_lib_rs_in_test_repo(&test_repo, "pub struct AddedSinceLatest;");

    let mut cmd = TestCmd::new().with_separate_target_dir();
    cmd.current_dir(test_repo.path());
    cmd.arg("diff");
    if let Some(arg) = arg {
        cmd.arg(arg);
    }
    cmd.assert()
        .stdout_with_insta("diff-latest")
        .stderr(contains(expected_stderr))
        .success();
}

#[test]
fn diff_published_explicit_package() {
    let mut cmd = TestCmd::new().with_test_repo();
    cmd.arg("-p");
    cmd.arg("example_api");
    cmd.arg("diff");
    cmd.arg("0.1.0");
    cmd.assert().stdout_with_insta("diff_published").success();
}

#[test]
fn diff_published_explicit_package_after_diff_subcommand() {
    let mut cmd = TestCmd::new().with_test_repo();
    cmd.arg("diff");
    cmd.arg("0.1.0");
    cmd.arg("-p");
    cmd.arg("example_api");
    cmd.assert().stdout_with_insta("diff_published").success();
}

#[test]
fn diff_published_with_all_features() {
    let mut cmd = TestCmd::new().with_test_repo();
    cmd.arg("-p");
    cmd.arg("example_api");
    cmd.arg("--all-features");
    cmd.arg("diff");
    cmd.arg("0.2.1");
    cmd.assert()
        .stdout_with_insta("diff_published_with_features")
        .success();
}

#[test]
fn diff_with_features_separated_by_comma() {
    let mut cmd = TestCmd::new().with_test_repo_variant(TestRepoVariant::Features);
    cmd.args(["--features", "feature_a,feature_b"]);
    cmd.args(["diff", "HEAD..HEAD"]);
    cmd.assert().stdout_with_insta("no_diff").success();
}

#[test]
fn diff_with_features_separated_by_space_in_single_arg() {
    let mut cmd = TestCmd::new().with_test_repo_variant(TestRepoVariant::Features);
    cmd.args(["--features", "feature_a feature_b"]);
    cmd.args(["diff", "HEAD..HEAD"]);
    cmd.assert().stdout_with_insta("no_diff").success();
}

/// Expected to fail with an error looking something like "error: none of the
/// selected packages contains these features: HEAD..HEAD, diff". But we don't
/// care exactly how the error looks.
///
/// We have the same behavior as regular cargo. This works (which we test in
/// diff_with_features_separated_by_space_in_single_arg()):
///
/// ```sh
/// cd test-apis/features
/// cargo build --features "feature_a feature_b"
/// ```
///
/// but this does not work (which we test in this test).
///
/// ```sh
/// cd test-apis/features
/// cargo build --features feature_a feature_b
/// ```
#[test]
fn diff_with_features_separated_by_space() {
    let mut cmd = TestCmd::new().with_test_repo_variant(TestRepoVariant::Features);
    cmd.args(["--features", "feature_a", "feature_b"]);
    cmd.args(["diff", "HEAD..HEAD"]);
    cmd.assert().failure();
}

#[test]
fn list_public_items_from_json_file() {
    // Create independent build dir so all tests can run in parallel
    let build_dir = tempdir().unwrap();

    let json_file = rustdoc_json_path_for_crate("../test-apis/example_api-v0.3.0", &build_dir);
    let mut cmd = TestCmd::new().with_separate_target_dir();
    cmd.arg("--rustdoc-json");
    cmd.arg(json_file);
    cmd.assert()
        .stdout_with_insta("example_api-v0.3.0")
        .success();
}

#[test]
fn verbose() {
    let mut cmd = TestCmd::new();
    cmd.arg("--manifest-path");
    cmd.arg("../test-apis/lint_error/Cargo.toml");
    cmd.arg("--verbose");
    cmd.assert()
        .stdout(contains("Processing \""))
        .stdout(contains("rustdoc JSON missing referenced item"))
        .success();
}

#[test]
fn short_help() {
    let mut cmd = TestCmd::new().with_separate_target_dir();
    cmd.arg("-h");
    cmd.assert().stdout_with_insta("short-help").success();
}

#[test]
fn short_diff_help() {
    let mut cmd = TestCmd::new().with_separate_target_dir();
    cmd.arg("diff");
    cmd.arg("-h");
    cmd.assert().stdout_with_insta("short-diff-help").success();
}

#[test]
fn short_completions_help() {
    let mut cmd = TestCmd::new().with_separate_target_dir();
    cmd.arg("completions");
    cmd.arg("-h");
    cmd.assert()
        .stdout_with_insta("short-completions-help")
        .success();
}

#[test]
fn long_help() {
    let mut cmd = TestCmd::new();
    cmd.arg("--help");
    cmd.assert().stdout_with_insta("long-help").success();
}

#[test]
fn long_completions_help() {
    let mut cmd = TestCmd::new();
    cmd.arg("completions");
    cmd.arg("--help");
    cmd.assert()
        .stdout_with_insta("long-completions-help")
        .success();
}

#[test]
fn long_diff_help() {
    let mut cmd = TestCmd::new();
    cmd.arg("diff");
    cmd.arg("--help");
    cmd.assert().stdout_with_insta("long-diff-help").success();
}

#[test]
fn long_help_wraps() {
    let max_allowed_line_length = 125; // 120 with some margin

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

/// Test zsh completion script generation.
///
/// NOTE: This test requires zsh to be installed on the system. It is installed
/// by default on macOS, and is generally either already installed on your Linux
/// distribution or very easy to install.
#[test]
#[cfg_attr(
    target_family = "windows",
    ignore = "zsh completion script not relevant for Windows"
)]
fn zsh_shell_completions() {
    // Create a temp `fpath` dir for for zsh completion scripts
    let zsh_fpath = tempdir().unwrap();

    // Generate zsh completion script for `cargo`
    let mut rustup = std::process::Command::new("rustup");
    rustup.args(["completions", "zsh", "cargo"]);
    std::fs::write(
        zsh_fpath.path().to_path_buf().join("_cargo"),
        rustup.output().unwrap().stdout,
    )
    .unwrap();

    // Generate zsh completion script for `cargo public-api`
    let mut cmd = TestCmd::as_subcommand_without_args();
    cmd.args(["completions", "zsh"]);
    std::fs::write(
        zsh_fpath.path().to_path_buf().join("_cargo-public-api"),
        &cmd.assert().success().get_output().stdout,
    )
    .unwrap();

    // Now make sure that the zsh completion actually works
    let mut cmd = Command::from_std(std::process::Command::new("zsh"));
    cmd.arg("-f");
    cmd.arg("tests/test-zsh-completions.zsh");
    cmd.arg(zsh_fpath.path());
    cmd.assert().success();
}

fn create_test_repo_with_dirty_git_tree() -> TestRepo {
    let test_repo = TestRepo::new();

    // Make the tree dirty by appending a comment to src/lib.rs
    append_to_lib_rs_in_test_repo(&test_repo, "// Make git tree dirty");

    test_repo
}

fn append_to_lib_rs_in_test_repo(test_repo: &TestRepo, to_append: &str) {
    let mut lib_rs_path = test_repo.path().to_owned();
    lib_rs_path.push("src/lib.rs");

    let mut lib_rs = OpenOptions::new().append(true).open(&lib_rs_path).unwrap();

    writeln!(lib_rs, "{to_append}").unwrap();
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
    let mut cmd = TestCmd::new().with_test_repo_variant(TestRepoVariant::Features);

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
        .stdout_with_insta(&format!("features-feat{features}.txt"))
        .success();
}

fn rustdoc_json_path_for_crate(test_crate: &str, target_dir: impl AsRef<Path>) -> PathBuf {
    rustdoc_json_builder_for_crate(test_crate, target_dir)
        .build()
        .unwrap()
}

fn rustdoc_json_builder_for_crate(
    test_crate: &str,
    target_dir: impl AsRef<Path>,
) -> rustdoc_json::Builder {
    rustdoc_json::Builder::default()
        .manifest_path(format!("{test_crate}/Cargo.toml"))
        .toolchain("nightly")
        .target_dir(target_dir)
        .quiet(true)
}

/// A git repository that lives during the duration of a test. Having each test
/// have its own git repository to test with makes tests runnable concurrently.
struct TestRepo {
    path: PathBuf,
}

#[derive(Default)]
enum TestRepoVariant {
    #[default]
    ExampleApi,
    LintError,
    Features,
}

impl TestRepo {
    fn new() -> Self {
        Self::new_with_variant(TestRepoVariant::default())
    }

    fn new_with_variant(variant: TestRepoVariant) -> Self {
        let tempdir = tempfile::tempdir().unwrap();
        let dirs_and_tags: &[(&str, &str)] = match variant {
            TestRepoVariant::ExampleApi => &[
                ("example_api-v0.1.0", "v0.1.0"),
                ("example_api-v0.1.1", "v0.1.1"),
                ("example_api-v0.2.0", "v0.2.0"),
                ("example_api-v0.3.0", "v0.3.0"),
            ],
            TestRepoVariant::LintError => &[("lint_error", "v0.1.0"), ("lint_error", "v0.1.1")],
            TestRepoVariant::Features => &[("features", "v0.1.0")],
        };
        create_test_git_repo::create_test_git_repo(tempdir.path(), dirs_and_tags);

        Self {
            path: tempdir.into_path(),
        }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TestRepo {
    fn drop(&mut self) {
        if env::var_os("CARGO_PUBLIC_API_PRESERVE_TEST_REPO").is_some() {
            println!(
                "DEBUG: NOT removing test repo at {:?}. If the test fails, you can `cd` into it and debug the failure",
                self.path()
            );
        } else {
            remove_dir_all::remove_dir_all(self.path()).unwrap();
        }
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
    /// increases parallelism of tests. Note that if `test_repo` is used, no
    /// separate `./target` dir is needed, since the `./target` dir inside the
    /// (newly crated) test repo can and will be used.
    ///
    /// Note: Tests are not completely independent even with one target-dir per
    /// test, because `cargo` also makes use of a global shared package cache
    /// lockfile:
    /// https://github.com/rust-lang/cargo/blob/ba607b23db8398723d659249d9abf5536bc322e5/src/cargo/util/config/mod.rs#L1733-L1738
    target_dir: Option<tempfile::TempDir>,
}

enum TestCmdType<'str> {
    Subcommand { toolchain: Option<&'str str> },
    Bin,
}

#[cfg(not(target_family = "windows"))]
fn cargo_with_toolchain(toolchain: &str) -> std::process::Command {
    let mut cmd = std::process::Command::new("cargo");
    cmd.arg(format!("+{toolchain}"));
    cmd
}

/// Workaround for [rustup 1.25: On Windows, nested cargo invocation with a
/// toolchain specified fails](https://github.com/rust-lang/rustup/issues/3036).
/// We only use it on Windows, because when possible we want this command to be
/// as similar as possible to what real users use. And most users do
/// ```bash
/// cargo +toolchain public-api
/// ```
/// rather than
/// ```bash
/// rustup run toolchain cargo public-api
/// ```
#[cfg(target_family = "windows")]
fn cargo_with_toolchain(toolchain: &str) -> std::process::Command {
    let mut cmd = std::process::Command::new("rustup");
    cmd.arg("run");
    cmd.arg(toolchain);
    cmd.arg("cargo");
    cmd
}

impl From<TestCmdType<'_>> for Command {
    fn from(cmd_type: TestCmdType) -> Self {
        match cmd_type {
            TestCmdType::Subcommand { toolchain } => {
                assert_cargo_public_api_in_path();

                let mut cargo_cmd = if let Some(toolchain) = toolchain {
                    cargo_with_toolchain(toolchain)
                } else {
                    std::process::Command::new("cargo")
                };
                cargo_cmd.arg("public-api");

                Command::from_std(cargo_cmd)
            }
            TestCmdType::Bin => Command::cargo_bin("cargo-public-api").unwrap(),
        }
    }
}

impl TestCmd {
    /// `cargo-public-api --simplified`
    fn new() -> Self {
        Self::new_impl(TestCmdType::Bin, true)
    }

    /// `cargo public-api --simplified`
    fn as_subcommand() -> Self {
        Self::new_impl(TestCmdType::Subcommand { toolchain: None }, true)
    }

    /// `cargo public-api`
    fn as_subcommand_without_args() -> Self {
        Self::new_impl(TestCmdType::Subcommand { toolchain: None }, false)
    }

    /// `cargo +toolchain public-api --simplified`
    /// Also installs the toolchain if it is not installed.
    fn with_proxy_toolchain(toolchain: &str) -> Self {
        rustup_toolchain::install(toolchain).unwrap();
        Self::new_impl(
            TestCmdType::Subcommand {
                toolchain: Some(toolchain),
            },
            true,
        )
    }

    /// Disable colors to make asserts on output insensitive to color codes.
    fn without_cargo_colors(mut self) -> Self {
        self.cmd.env("CARGO_TERM_COLOR", "never");
        self
    }

    fn new_impl(cmd_type: TestCmdType, simplified: bool) -> Self {
        let mut cmd: Command = cmd_type.into();

        if simplified {
            // Simplify output since if we render all other items properly, the
            // risk is very low that we will render Blanket Implementations and
            // Auto Trait Implementations items wrong. Instead we choose to have
            // dedicated tests for the rendering of such items. NOTE: Here we
            // also test that ',' is a valid delimiter for `--omit`.
            cmd.args(["--omit", "blanket-impls,auto-trait-impls"]);
        }

        Self {
            cmd,
            test_repo: None,
            target_dir: None,
        }
    }

    fn with_test_repo(self) -> Self {
        Self::with_test_repo_variant(self, TestRepoVariant::default())
    }

    /// Create a test repo (unique for the current test) and set its dir as the
    /// current dir.
    fn with_test_repo_variant(mut self, variant: TestRepoVariant) -> Self {
        let test_repo = TestRepo::new_with_variant(variant);
        self.cmd.current_dir(test_repo.path());
        self.test_repo = Some(test_repo);

        // Use a separate target dir even if we have a test repo with its own
        // ./target dir. Because when we run diff <version> tests, they will
        // share `.cargo-lock` (via `build-root-for-published-crates`)
        // otherwise.
        self.with_separate_target_dir()
    }

    /// Setup a separate target dir for the test. Helps with parallelism.
    fn with_separate_target_dir(mut self) -> Self {
        let target_dir = tempfile::tempdir().unwrap();
        self.cmd.arg("--target-dir").arg(target_dir.path());
        self.target_dir = Some(target_dir);
        self
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

    pub fn cmd(&mut self) -> &mut Command {
        &mut self.cmd
    }

    pub fn assert(&mut self) -> Assert {
        self.cmd.assert()
    }
}

struct RmOnDrop(PathBuf);

impl Drop for RmOnDrop {
    fn drop(&mut self) {
        std::fs::remove_file(&self.0).unwrap();
    }
}

pub trait AssertOrUpdate {
    fn stdout_with_insta(self, test_name: &str) -> Assert;
}

impl AssertOrUpdate for Assert {
    fn stdout_with_insta(self, test_name: &str) -> Assert {
        // Before assert stdout, make sure we didn't fail to parse rustdoc JSON,
        // which is common if rustdoc-types crate is updated, and if that is the
        // case we want to see stderr to get a clue why parsing failed.
        let stderr = String::from_utf8_lossy(&self.get_output().stderr);
        assert!(
            !stderr.contains("Failed to parse rustdoc JSON"),
            "Failed to parse rustdoc JSON. Stderr:\n{stderr}"
        );

        let stdout = String::from_utf8_lossy(&self.get_output().stdout);
        snapshot_testing::assert_eq_or_update(stdout, format!("tests/snapshots/{test_name}.txt"));
        self
    }
}

/// Figures out the `./target/debug` dir
fn bin_dir() -> PathBuf {
    let mut bin_dir = env::current_exe().unwrap(); // ".../target/debug/deps/cargo_public_api_bin_tests-d0f2f926b349fbb9"
    bin_dir.pop(); // Pop "cargo_public_api_bin_tests-d0f2f926b349fbb9"
    bin_dir.pop(); // Pop "deps"
    bin_dir // ".../target/debug"
}

/// Since `rustup` always prepends `$CARGO_HOME/bin` to `$PATH` [1], make sure
/// `cargo-public-api` is not there, so that tests will use the freshly built
/// `cargo-public-api` rather than something old. The best way to keep
/// `cargo-public-api` in your PATH without interfering with tests is to rename
/// it. See what command to use in the assertion message below.
///
/// [1]
/// <https://github.com/rust-lang/rustup/blob/a223e5ad6549e5fb0c56932fd0e79af9de898ad4/src/toolchain.rs#L446-L453>
fn assert_cargo_public_api_not_in_cargo_home_bin() {
    let Some(mut path) = home::cargo_home().ok() else {
        return;
    };

    path.push("bin");
    path.push(format!("cargo-public-api{EXE_SUFFIX}"));

    assert!(
        std::fs::metadata(&path).is_err(),
        "Found {path:?} which will override `./target/debug/cargo-public-api`
and thus interfere with tests. Run

    mv -v ~/.cargo/bin/cargo-public-api ~/.cargo/bin/$(~/.cargo/bin/cargo-public-api --version | tr ' ' '-')

to fix.");
}

fn assert_cargo_public_api_in_path() {
    assert_cargo_public_api_not_in_cargo_home_bin();

    let path = env::var_os("PATH").unwrap();
    assert!(
        env::split_paths(&path)
            .map(|path| path.join(format!("cargo-public-api{EXE_SUFFIX}")))
            .any(|path| path.exists()),
        "Could not find `cargo-public-api` in `PATH` which is set to:

{path:?}

Make sure that `./target/debug` is in `PATH`:

    PATH=\"$(pwd)/target/debug:$PATH\" cargo test

or run the helper script that does that for you:

    ./scripts/cargo-test.sh

"
    );
}

/// See where this is used for an explanation of why we have this helper.
fn get_toolchain_one_day_before_minimal_toolchain() -> String {
    nightly_version_minus_one_day(MINIMUM_NIGHTLY_RUST_VERSION)
}

/// Convert e.g. `nightly-2023-01-23` to `nightly-2023-01-22`, i.e. minus a day.
fn nightly_version_minus_one_day(nightly_version: impl AsRef<str>) -> String {
    nightly_version_from_date(
        date_from_nightly_version(nightly_version)
            .checked_sub(1.day())
            .unwrap(),
    )
}

fn date_from_nightly_version(nightly_version: impl AsRef<str>) -> Date {
    let date = nightly_version
        .as_ref()
        .strip_prefix("nightly-")
        .expect("nightly version starts with 'nightly-'");
    date.parse()
        .expect("nightly version is in 'YYYY-MM-DD' format")
}

fn nightly_version_from_date(date: Date) -> String {
    format!("nightly-{date}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nightly_version_minus_one_day() {
        let cases = [
            ("nightly-2023-01-01", "nightly-2022-12-31"),
            ("nightly-2023-01-02", "nightly-2023-01-01"),
            ("nightly-2023-11-02", "nightly-2023-11-01"),
            ("nightly-2023-11-01", "nightly-2023-10-31"),
            ("nightly-2023-11-01", "nightly-2023-10-31"),
        ];
        for case in cases {
            assert_eq!(nightly_version_minus_one_day(case.0), case.1);
        }
    }
}
