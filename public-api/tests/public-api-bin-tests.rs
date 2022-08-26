// deny in CI, only warn here
#![warn(clippy::all, clippy::pedantic)]

use std::{io::BufRead, str::from_utf8};

use assert_cmd::Command;
use public_api::MINIMUM_RUSTDOC_JSON_VERSION;

mod utils;
use utils::rustdoc_json_path_for_crate;

#[test]
fn print_public_api() {
    cmd_with_rustdoc_json_args(&["../test-apis/comprehensive_api"], |mut cmd| {
        cmd.assert()
            .stdout(include_str!("./expected-output/comprehensive_api.txt"))
            .stderr("")
            .success();
    });
}

#[test]
fn print_public_api_with_blanket_implementations() {
    cmd_with_rustdoc_json_args(&["../test-apis/example_api-v0.2.0"], |mut cmd| {
        cmd.arg("--with-blanket-implementations");
        cmd.assert()
            .stdout(include_str!(
                "./expected-output/example_api-v0.2.0-with-blanket-implementations.txt"
            ))
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
                .stdout(
                    "Removed:
(nothing)

Changed:
-pub fn example_api::function(v1_param: Struct)
+pub fn example_api::function(v1_param: Struct, v2_param: usize)
-pub struct example_api::Struct
+#[non_exhaustive] pub struct example_api::Struct

Added:
+pub struct example_api::StructV2
+pub struct field example_api::Struct::v2_field: usize
+pub struct field example_api::StructV2::field: usize

",
                )
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
                .stdout(
                    "Removed:
-pub struct example_api::StructV2
-pub struct field example_api::Struct::v2_field: usize
-pub struct field example_api::StructV2::field: usize

Changed:
-#[non_exhaustive] pub struct example_api::Struct
+pub struct example_api::Struct
-pub fn example_api::function(v1_param: Struct, v2_param: usize)
+pub fn example_api::function(v1_param: Struct)

Added:
(nothing)

",
                )
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
                .stdout(
                    "Removed:
(nothing)

Changed:
(nothing)

Added:
(nothing)

",
                )
                .stderr("")
                .success();
        },
    );
}

/// Uses a bash one-liner to test that public-api gracefully handles
/// `std::io::ErrorKind::BrokenPipe`
#[test]
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

    for crate_ in crates {
        cmd.arg(rustdoc_json_path_for_crate(crate_));
    }

    final_steps(cmd);
}
