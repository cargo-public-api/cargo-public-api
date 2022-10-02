// deny in CI, only warn here
#![warn(clippy::all, clippy::pedantic)]

use std::{io::BufRead, str::from_utf8};

use assert_cmd::Command;
use public_api::MINIMUM_RUSTDOC_JSON_VERSION;

// rust-analyzer bug: https://github.com/rust-lang/rust-analyzer/issues/9173
#[path = "../../test-utils/src/lib.rs"]
mod test_utils;
use tempfile::TempDir;
use test_utils::assert_or_bless::AssertOrBless;
use test_utils::rustdoc_json_path_for_crate;
use test_utils::rustdoc_json_path_for_crate_with_target_dir;

#[test]
fn print_public_api() {
    cmd_with_rustdoc_json_args(&["../test-apis/comprehensive_api"], |mut cmd| {
        cmd.assert()
            .stdout_or_bless("./tests/expected-output/comprehensive_api.txt")
            .stderr("")
            .success();
    });
}

#[test]
fn print_public_api_with_blanket_implementations() {
    cmd_with_rustdoc_json_args(&["../test-apis/example_api-v0.2.0"], |mut cmd| {
        cmd.arg("--with-blanket-implementations");
        cmd.assert()
            .stdout_or_bless(
                "./tests/expected-output/example_api-v0.2.0-with-blanket-implementations.txt",
            )
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
                .stdout_or_bless("./tests/expected-output/print_diff.txt")
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
                .stdout_or_bless("./tests/expected-output/print_diff_reversed.txt")
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
                .stdout_or_bless("./tests/expected-output/print_no_diff.txt")
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
    // Use the JSON for a somewhat large API so the pipe has time to become closed
    // before all output has been written to stdout
    let large_api = rustdoc_json_path_for_crate("../test-apis/comprehensive_api");

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
        .stdout(format!("{}\n", MINIMUM_RUSTDOC_JSON_VERSION))
        .stderr("")
        .success();
}

#[test]
fn too_many_args_shows_help() {
    let mut cmd = Command::cargo_bin("public-api").unwrap();
    cmd.args(&["too", "many", "args"]);
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

To include blanket implementations, pass --with-blanket-implementations.

",
        env!("CARGO_PKG_VERSION"),
        MINIMUM_RUSTDOC_JSON_VERSION,
    )
}

/// Helper to setup a `public-api` [`Command`] with rustdoc JSON path args
/// corresponding to the given crates. Use `final_steps` to specify the
/// remaining steps in the test.
fn cmd_with_rustdoc_json_args(crates: &[&str], final_steps: impl FnOnce(Command)) {
    let mut cmd = Command::cargo_bin("public-api").unwrap();

    let mut temp_dirs = vec![];

    for crate_ in crates {
        // Put output in a temp dir to ensure concurrently running tests do not
        // write and read the same JSON simultaneously, which causes tests to fail
        // sporadically.
        let temp_dir = TempDir::new().unwrap();

        cmd.arg(rustdoc_json_path_for_crate_with_target_dir(
            crate_,
            temp_dir.path(),
        ));

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
