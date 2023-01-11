// deny in CI, only warn here
#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::disallowed_methods)]

use std::path::Path;
use std::{io::BufRead, str::from_utf8};

use assert_cmd::assert::Assert;
use assert_cmd::Command;
use public_api::MINIMUM_RUSTDOC_JSON_VERSION;

use tempfile::{tempdir, TempDir};

mod common;
use common::rustdoc_json_path_for_crate;

#[test]
fn print_public_api() {
    cmd_with_rustdoc_json_args(&["../test-apis/comprehensive_api"], |mut cmd| {
        cmd.assert()
            .stdout_or_update("./expected-output/comprehensive_api.txt")
            .stderr("")
            .success();
    });
}

#[test]
fn print_public_api_not_simplified() {
    cmd_with_rustdoc_json_args_not_simplified(&["../test-apis/example_api-v0.2.0"], |mut cmd| {
        cmd.assert()
            .stdout_or_update("./expected-output/example_api-v0.2.0-not-simplified.txt")
            .stderr("")
            .success();
    });
}

#[test]
fn print_diff() {
    cmd_with_rustdoc_json_args(
        &[
            "../test-apis/example_api-v0.1.0",
            "../test-apis/example_api-v0.2.0",
        ],
        |mut cmd| {
            cmd.assert()
                .stdout_or_update("./expected-output/print_diff.txt")
                .stderr("")
                .success();
        },
    );
}

#[test]
fn print_diff_reversed() {
    cmd_with_rustdoc_json_args(
        &[
            "../test-apis/example_api-v0.2.0",
            "../test-apis/example_api-v0.1.0",
        ],
        |mut cmd| {
            cmd.assert()
                .stdout_or_update("./expected-output/print_diff_reversed.txt")
                .stderr("")
                .success();
        },
    );
}

#[test]
fn print_no_diff() {
    cmd_with_rustdoc_json_args(
        &[
            "../test-apis/example_api-v0.2.0",
            "../test-apis/example_api-v0.2.0",
        ],
        |mut cmd| {
            cmd.assert()
                .stdout_or_update("./expected-output/print_no_diff.txt")
                .stderr("")
                .success();
        },
    );
}

/// Uses a bash one-liner to test that public-api gracefully handles
/// `std::io::ErrorKind::BrokenPipe`
#[test]
#[cfg_attr(target_family = "windows", ignore)] // Because test uses bash
fn broken_pipe() {
    // Create independent build dir so all tests can run in parallel
    let build_dir = tempdir().unwrap();

    // Use the JSON for a somewhat large API so the pipe has time to become closed
    // before all output has been written to stdout
    let large_api = rustdoc_json_path_for_crate("../test-apis/comprehensive_api", &build_dir);

    // Now setup the actual one-liner
    let mut cmd = std::process::Command::new("bash");
    cmd.args([
        "-c",
        &format!(
            "../target/debug/public-api {} | head -n 1",
            large_api.to_string_lossy(),
        ),
    ]);

    // Run it and assert on that there was no error printed
    assert_eq!(cmd.output().unwrap().stdout.lines().count(), 1);
    assert_eq!(from_utf8(&cmd.output().unwrap().stderr), Ok(""));
}

#[test]
fn short_help() {
    let mut cmd = Command::cargo_bin("public-api").unwrap();
    cmd.arg("-h");
    cmd.assert()
        .stdout(expected_help_text())
        .stderr("")
        .success();
}

#[test]
fn long_help() {
    let mut cmd = Command::cargo_bin("public-api").unwrap();
    cmd.arg("--help");
    cmd.assert()
        .stdout(expected_help_text())
        .stderr("")
        .success();
}

#[test]
fn no_args_shows_help() {
    let mut cmd = Command::cargo_bin("public-api").unwrap();
    cmd.assert()
        .stdout(expected_help_text())
        .stderr("")
        .success();
}

#[test]
fn print_minimum_rustdoc_json_version() {
    let mut cmd = Command::cargo_bin("public-api").unwrap();
    cmd.arg("--print-minimum-rustdoc-json-version");
    cmd.assert()
        .stdout(format!("{MINIMUM_RUSTDOC_JSON_VERSION}\n"))
        .stderr("")
        .success();
}

#[test]
fn too_many_args_shows_help() {
    let mut cmd = Command::cargo_bin("public-api").unwrap();
    cmd.args(["too", "many", "args"]);
    cmd.assert()
        .stdout(expected_help_text())
        .stderr("")
        .success();
}

fn expected_help_text() -> String {
    format!(
        "public-api v{}

Requires at least {}.

NOTE: See https://github.com/Enselic/cargo-public-api for a convenient cargo
wrapper around this program (or to be precise; library) that does everything
automatically.

If you insist of using this low-level utility and thin wrapper, you run it like this:

    public-api <RUSTDOC_JSON_FILE>

where RUSTDOC_JSON_FILE is the path to the output of

    cargo +nightly rustdoc --lib -- -Z unstable-options --output-format json

which you can find in

    ./target/doc/${{CRATE}}.json

To diff the public API between two commits, you generate one rustdoc JSON file for each
commit and then pass the path of both files to this utility:

    public-api <RUSTDOC_JSON_FILE_OLD> <RUSTDOC_JSON_FILE_NEW>


",
        env!("CARGO_PKG_VERSION"),
        MINIMUM_RUSTDOC_JSON_VERSION,
    )
}

fn cmd_with_rustdoc_json_args(crates: &[&str], final_steps: impl FnOnce(Command)) {
    cmd_with_rustdoc_json_args_impl(crates, true, final_steps);
}

fn cmd_with_rustdoc_json_args_not_simplified(crates: &[&str], final_steps: impl FnOnce(Command)) {
    cmd_with_rustdoc_json_args_impl(crates, false, final_steps);
}

/// Helper to setup a `public-api` [`Command`] with rustdoc JSON path args
/// corresponding to the given crates. Use `final_steps` to specify the
/// remaining steps in the test.
fn cmd_with_rustdoc_json_args_impl(
    crates: &[&str],
    simplified: bool,
    final_steps: impl FnOnce(Command),
) {
    let mut cmd = Command::cargo_bin("public-api").unwrap();
    if simplified {
        cmd.arg("--simplified");
    }

    let mut temp_dirs = vec![];

    for crate_ in crates {
        // Put output in a temp dir to ensure concurrently running tests do not
        // write and read the same JSON simultaneously, which causes tests to fail
        // sporadically.
        let temp_dir = TempDir::new().unwrap();

        cmd.arg(rustdoc_json_path_for_crate(crate_, temp_dir.path()));

        // We need one dir per crate, because in the case of example_api-v0.1.0 and
        // example_api-v0.2.0 for example, the same JSON file name example_api.json
        // is used, so using the same dir for all would cause JSON files to be
        // overwritten.
        temp_dirs.push(temp_dir);
    }

    final_steps(cmd);

    // Here temp_dirs are dropped/removed. To prevent that for debugging
    // purposes, use `TempDir::into_path()`.
}

pub trait AssertOrUpdate {
    fn stdout_or_update(self, expected_file: impl AsRef<Path>) -> Assert;
}

impl AssertOrUpdate for Assert {
    fn stdout_or_update(self, expected_file: impl AsRef<Path>) -> Assert {
        let stdout = String::from_utf8_lossy(&self.get_output().stdout);
        expect_test::expect_file![expected_file.as_ref()].assert_eq(&stdout);
        self
    }
}
