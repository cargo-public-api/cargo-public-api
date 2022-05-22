use std::path::{Path, PathBuf};

use assert_cmd::Command;
use serial_test::serial;

#[test]
fn list_public_items() {
    let cmd = Command::cargo_bin("cargo-public-api").unwrap();
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
fn virtual_manifest_error() {
    let mut cmd = Command::cargo_bin("cargo-public-api").unwrap();
    cmd.arg("--manifest-path");
    cmd.arg(current_dir_and("tests/virtual-manifest/Cargo.toml"));
    cmd.assert()
        .stdout("")
        .stderr(predicates::str::contains(
            "Listing or diffing the public API of an entire workspace is not supported.",
        ))
        .failure();
}

// We must serially run tests that touch the test crate git repo to prevent
// ".git/index.lock: File exists"-errors.
#[serial]
#[test]
fn diff_public_items() {
    ensure_test_crate_is_cloned();

    let mut cmd = Command::cargo_bin("cargo-public-api").unwrap();
    cmd.current_dir(test_crate_path());
    cmd.arg("--color=never");
    cmd.arg("--diff-git-checkouts");
    cmd.arg("v0.0.4");
    cmd.arg("v0.0.5");
    cmd.assert()
        .stdout(
            "Removed items from the public API\n\
             =================================\n\
             -pub fn public_items::from_rustdoc_json_str(rustdoc_json_str: &str) -> Result<HashSet<String>>\n\
             \n\
             Changed items in the public API\n\
             ===============================\n\
             (none)\n\
             \n\
             Added items to the public API\n\
             =============================\n\
             +pub fn public_items::sorted_public_items_from_rustdoc_json_str(rustdoc_json_str: &str) -> Result<Vec<String>>\n\
             \n\
            ",
        )
        .success();
}

#[serial]
#[test]
fn diff_public_items_with_color() {
    ensure_test_crate_is_cloned();

    let mut cmd = Command::cargo_bin("cargo-public-api").unwrap();
    cmd.current_dir(test_crate_path());
    cmd.arg("--color=always");
    cmd.arg("--diff-git-checkouts");
    cmd.arg("v0.6.0");
    cmd.arg("v0.7.1");

    // To update expected output, run this:
    //
    //   cd target/tmp/cargo-public-api-test-repo
    //   cargo run --manifest-path ../../../Cargo.toml -- --color=always --diff-git-checkouts v0.6.0 v0.7.1 | sed 's/\x1b/\\x1b/g'
    cmd.assert()
        .stdout("Removed items from the public API
=================================
-\x1b[34mpub\x1b[0m \x1b[34mfn\x1b[0m \x1b[36mpublic_items\x1b[0m::\x1b[32mPublicItem\x1b[0m::\x1b[33mhash\x1b[0m<\x1b[32m__H\x1b[0m: \x1b[36m$crate\x1b[0m::\x1b[36mhash\x1b[0m::\x1b[32mHasher\x1b[0m>(&\x1b[34mself\x1b[0m, \x1b[36mstate\x1b[0m: &\x1b[34mmut\x1b[0m \x1b[32m__H\x1b[0m) -> ()
-\x1b[34mpub\x1b[0m \x1b[34mfn\x1b[0m \x1b[36mpublic_items\x1b[0m::\x1b[36mdiff\x1b[0m::\x1b[32mPublicItemsDiff\x1b[0m::\x1b[33mprint_with_headers\x1b[0m(&\x1b[34mself\x1b[0m, \x1b[36mw\x1b[0m: &\x1b[34mmut\x1b[0m \x1b[34mimpl\x1b[0m \x1b[36mstd\x1b[0m::\x1b[36mio\x1b[0m::\x1b[32mWrite\x1b[0m, \x1b[36mheader_removed\x1b[0m: &\x1b[32mstr\x1b[0m, \x1b[36mheader_changed\x1b[0m: &\x1b[32mstr\x1b[0m, \x1b[36mheader_added\x1b[0m: &\x1b[32mstr\x1b[0m) -> \x1b[36mstd\x1b[0m::\x1b[36mio\x1b[0m::\x1b[32mResult\x1b[0m<()>

Changed items in the public API
===============================
-\x1b[34mpub\x1b[0m \x1b[34mfn\x1b[0m \x1b[36mpublic_items\x1b[0m::\x1b[32mPublicItem\x1b[0m::\x1b[33mfmt\x1b[0m(&\x1b[34mself\x1b[0m, \x1b[36mf\x1b[0m: &\x1b[34mmut\x1b[0m \x1b[36m$crate\x1b[0m::\x1b[36mfmt\x1b[0m::\x1b[32mFormatter\x1b[0m<\x1b[34m'_\x1b[0m>) -> \x1b[36m$crate\x1b[0m::\x1b[36mfmt\x1b[0m::\x1b[32mResult\x1b[0m
+\x1b[34mpub\x1b[0m \x1b[34mfn\x1b[0m \x1b[36mpublic_items\x1b[0m::\x1b[32mPublicItem\x1b[0m::\x1b[33mfmt\x1b[0m(&\x1b[34mself\x1b[0m, \x1b[36mf\x1b[0m: &\x1b[34mmut\x1b[0m \x1b[36mstd\x1b[0m::\x1b[36mfmt\x1b[0m::\x1b[32mFormatter\x1b[0m<\x1b[34m'_\x1b[0m>) -> \x1b[36mstd\x1b[0m::\x1b[36mfmt\x1b[0m::\x1b[32mResult\x1b[0m
-\x1b[34mpub\x1b[0m \x1b[34mfn\x1b[0m \x1b[36mpublic_items\x1b[0m::\x1b[36mdiff\x1b[0m::\x1b[32mPublicItemsDiff\x1b[0m::\x1b[33mbetween\x1b[0m(\x1b[36mold\x1b[0m: \x1b[32mVec\x1b[0m<\x1b[32mPublicItem\x1b[0m>, \x1b[36mnew\x1b[0m: \x1b[32mVec\x1b[0m<\x1b[32mPublicItem\x1b[0m>) -> \x1b[32mSelf\x1b[0m
+\x1b[34mpub\x1b[0m \x1b[34mfn\x1b[0m \x1b[36mpublic_items\x1b[0m::\x1b[36mdiff\x1b[0m::\x1b[32mPublicItemsDiff\x1b[0m::\x1b[33mbetween\x1b[0m(\x1b[36mold_items\x1b[0m: \x1b[32mVec\x1b[0m<\x1b[32mPublicItem\x1b[0m>, \x1b[36mnew_items\x1b[0m: \x1b[32mVec\x1b[0m<\x1b[32mPublicItem\x1b[0m>) -> \x1b[32mSelf\x1b[0m

Added items to the public API
=============================
+\x1b[34mpub\x1b[0m \x1b[34mfn\x1b[0m \x1b[36mpublic_items\x1b[0m::\x1b[36mdiff\x1b[0m::\x1b[32mChangedPublicItem\x1b[0m::\x1b[33mcmp\x1b[0m(&\x1b[34mself\x1b[0m, \x1b[36mother\x1b[0m: &\x1b[32mChangedPublicItem\x1b[0m) -> \x1b[36m$crate\x1b[0m::\x1b[36mcmp\x1b[0m::\x1b[32mOrdering\x1b[0m
+\x1b[34mpub\x1b[0m \x1b[34mfn\x1b[0m \x1b[36mpublic_items\x1b[0m::\x1b[36mdiff\x1b[0m::\x1b[32mChangedPublicItem\x1b[0m::\x1b[33meq\x1b[0m(&\x1b[34mself\x1b[0m, \x1b[36mother\x1b[0m: &\x1b[32mChangedPublicItem\x1b[0m) -> \x1b[32mbool\x1b[0m
+\x1b[34mpub\x1b[0m \x1b[34mfn\x1b[0m \x1b[36mpublic_items\x1b[0m::\x1b[36mdiff\x1b[0m::\x1b[32mChangedPublicItem\x1b[0m::\x1b[33mne\x1b[0m(&\x1b[34mself\x1b[0m, \x1b[36mother\x1b[0m: &\x1b[32mChangedPublicItem\x1b[0m) -> \x1b[32mbool\x1b[0m
+\x1b[34mpub\x1b[0m \x1b[34mfn\x1b[0m \x1b[36mpublic_items\x1b[0m::\x1b[36mdiff\x1b[0m::\x1b[32mChangedPublicItem\x1b[0m::\x1b[33mpartial_cmp\x1b[0m(&\x1b[34mself\x1b[0m, \x1b[36mother\x1b[0m: &\x1b[32mChangedPublicItem\x1b[0m) -> \x1b[36m$crate\x1b[0m::\x1b[36moption\x1b[0m::\x1b[32mOption\x1b[0m<\x1b[36m$crate\x1b[0m::\x1b[36mcmp\x1b[0m::\x1b[32mOrdering\x1b[0m>
+\x1b[34mpub\x1b[0m \x1b[34mfn\x1b[0m \x1b[36mpublic_items\x1b[0m::\x1b[36mdiff\x1b[0m::\x1b[32mPublicItemsDiff\x1b[0m::\x1b[33meq\x1b[0m(&\x1b[34mself\x1b[0m, \x1b[36mother\x1b[0m: &\x1b[32mPublicItemsDiff\x1b[0m) -> \x1b[32mbool\x1b[0m
+\x1b[34mpub\x1b[0m \x1b[34mfn\x1b[0m \x1b[36mpublic_items\x1b[0m::\x1b[36mdiff\x1b[0m::\x1b[32mPublicItemsDiff\x1b[0m::\x1b[33mne\x1b[0m(&\x1b[34mself\x1b[0m, \x1b[36mother\x1b[0m: &\x1b[32mPublicItemsDiff\x1b[0m) -> \x1b[32mbool\x1b[0m

",
        )
        .success();
}

#[serial]
#[test]
fn list_public_items_with_color() {
    ensure_test_crate_is_cloned();

    let mut cmd = Command::cargo_bin("cargo-public-api").unwrap();
    cmd.arg("--color=always");

    // To update expected output, run this:
    //
    //   cargo run -- --color=always | sed 's/\x1b/\\x1b/g'
    cmd.assert()
        .stdout("\x1b[34mpub\x1b[0m \x1b[34mfn\x1b[0m \x1b[36mcargo_public_api\x1b[0m::\x1b[33mfor_self_testing_purposes_please_ignore\x1b[0m()
\x1b[34mpub\x1b[0m \x1b[34mmod\x1b[0m \x1b[36mcargo_public_api\x1b[0m
",
        )
        .success();
}

#[serial]
#[test]
fn diff_public_items_markdown() {
    ensure_test_crate_is_cloned();

    let mut cmd = Command::cargo_bin("cargo-public-api").unwrap();
    cmd.current_dir(test_crate_path());
    cmd.arg("--output-format=markdown");
    cmd.arg("--diff-git-checkouts");
    cmd.arg("v0.6.0");
    cmd.arg("v0.7.1");
    cmd.assert()
        .stdout(r"## Removed items from the public API
* `pub fn public_items::PublicItem::hash<__H: $crate::hash::Hasher>(&self, state: &mut __H) -> ()`
* `pub fn public_items::diff::PublicItemsDiff::print_with_headers(&self, w: &mut impl std::io::Write, header_removed: &str, header_changed: &str, header_added: &str) -> std::io::Result<()>`

## Changed items in the public API
* `pub fn public_items::PublicItem::fmt(&self, f: &mut $crate::fmt::Formatter<'_>) -> $crate::fmt::Result` changed to
  `pub fn public_items::PublicItem::fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result`
* `pub fn public_items::diff::PublicItemsDiff::between(old: Vec<PublicItem>, new: Vec<PublicItem>) -> Self` changed to
  `pub fn public_items::diff::PublicItemsDiff::between(old_items: Vec<PublicItem>, new_items: Vec<PublicItem>) -> Self`

## Added items to the public API
* `pub fn public_items::diff::ChangedPublicItem::cmp(&self, other: &ChangedPublicItem) -> $crate::cmp::Ordering`
* `pub fn public_items::diff::ChangedPublicItem::eq(&self, other: &ChangedPublicItem) -> bool`
* `pub fn public_items::diff::ChangedPublicItem::ne(&self, other: &ChangedPublicItem) -> bool`
* `pub fn public_items::diff::ChangedPublicItem::partial_cmp(&self, other: &ChangedPublicItem) -> $crate::option::Option<$crate::cmp::Ordering>`
* `pub fn public_items::diff::PublicItemsDiff::eq(&self, other: &PublicItemsDiff) -> bool`
* `pub fn public_items::diff::PublicItemsDiff::ne(&self, other: &PublicItemsDiff) -> bool`

",
        )
        .success();
}

#[serial]
#[test]
fn diff_public_items_markdown_no_changes() {
    ensure_test_crate_is_cloned();

    let mut cmd = Command::cargo_bin("cargo-public-api").unwrap();
    cmd.current_dir(test_crate_path());
    cmd.arg("--output-format=markdown");
    cmd.arg("--diff-git-checkouts");
    cmd.arg("v0.2.0");
    cmd.arg("v0.3.0");
    cmd.assert()
        .stdout("(No changes to the public API)\n")
        .success();
}

#[serial]
#[test]
fn diff_public_items_from_files_no_changes() {
    // Build ./target/doc/cargo_public_api.json
    let mut cmd = Command::cargo_bin("cargo-public-api").unwrap();
    cmd.arg("--only-build-rustdoc-json");
    cmd.assert().success();

    // Just a sanity check that --diff-rustdoc-json works at all
    let mut cmd = Command::cargo_bin("cargo-public-api").unwrap();
    cmd.arg("--diff-rustdoc-json");
    cmd.arg("./target/doc/cargo_public_api.json");
    cmd.arg("./target/doc/cargo_public_api.json");
    cmd.assert()
        .stdout(
            "Removed items from the public API
=================================
(none)

Changed items in the public API
===============================
(none)

Added items to the public API
=============================
(none)

",
        )
        .success();
}

#[serial]
#[test]
fn diff_public_items_missing_one_arg() {
    let mut cmd = Command::cargo_bin("cargo-public-api").unwrap();
    cmd.current_dir(test_crate_path());
    cmd.arg("--diff-git-checkouts");
    cmd.arg("v0.2.0");
    cmd.assert()
        .stderr(predicates::str::contains(
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
             \n\
             ",
        )
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
             ",
        )
        .success();
}

fn assert_presence_of_args_in_help(mut cmd: Command) {
    cmd.assert()
        .stdout(predicates::str::contains("--with-blanket-implementations"))
        .stdout(predicates::str::contains("--manifest-path"))
        .stdout(predicates::str::contains("--diff-git-checkouts"))
        .success();
}

fn ensure_test_crate_is_cloned() {
    let path = test_crate_path();
    if path.exists() {
        print!("INFO: using ");
    } else {
        print!("INFO: cloning into ");
        clone_test_crate(&path);
    }
    // Print info about repo when running like this: cargo test -- --nocapture
    println!("'{}'", &path.to_string_lossy());
}

/// Helper to get the absolute path to a given path, relative to the current
/// path
fn current_dir_and<P: AsRef<Path>>(path: P) -> PathBuf {
    let mut cur_dir = std::env::current_dir().unwrap();
    cur_dir.push(path);
    cur_dir
}

/// Helper to clone the test crate git repo to the proper place
fn clone_test_crate(dest: &Path) {
    let mut git = std::process::Command::new("git");
    git.arg("clone");
    git.arg("https://github.com/Enselic/public-api.git"); // Tests still use ld name `public_items`
    git.arg("-b");
    git.arg("v0.7.1");
    git.arg("--single-branch");
    git.arg(dest);
    assert!(git.spawn().unwrap().wait().unwrap().success());
}

/// Path to the git cloned test crate we use to test the diffing functionality
fn test_crate_path() -> PathBuf {
    let mut path = get_cache_dir();
    path.push("cargo-public-api-test-repo");
    path
}

/// Where to put things that survives across test runs. For example a git cloned
/// test crate. We don't want to clone it every time we run tests.
fn get_cache_dir() -> PathBuf {
    // See https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-crates
    option_env!("CARGO_TARGET_TMPDIR")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
}
