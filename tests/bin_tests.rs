use assert_cmd::Command;
use public_items::MINIMUM_RUSTDOC_JSON_VERSION;

mod utils;
use serial_test::serial;
use utils::rustdoc_json_path_for_crate;

#[test]
#[serial] // Writing and reading rustdoc JSON to/from file-system; must run one test at a time
fn print_public_items() {
    cmd_with_rustdoc_json_args(&["./tests/crates/comprehensive_api"], |mut cmd| {
        cmd.assert()
            .stdout(include_str!("./expected-output/comprehensive_api.txt"))
            .stderr("")
            .success();
    });
}

#[test]
#[serial]
fn print_public_items_with_blanket_implementations() {
    cmd_with_rustdoc_json_args(&["./tests/crates/example_api-v0.2.0"], |mut cmd| {
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
#[serial]
fn print_diff() {
    cmd_with_rustdoc_json_args(
        &[
            "./tests/crates/example_api-v0.1.0",
            "./tests/crates/example_api-v0.2.0",
        ],
        |mut cmd| {
            cmd.assert()
                .stdout(
                    "Removed:
(nothing)

Changed:
-pub fn example_api::function(v1_param: Struct)
+pub fn example_api::function(v1_param: Struct, v2_param: usize)

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
#[serial]
fn print_diff_reversed() {
    cmd_with_rustdoc_json_args(
        &[
            "./tests/crates/example_api-v0.2.0",
            "./tests/crates/example_api-v0.1.0",
        ],
        |mut cmd| {
            cmd.assert()
                .stdout(
                    "Removed:
-pub struct example_api::StructV2
-pub struct field example_api::Struct::v2_field: usize
-pub struct field example_api::StructV2::field: usize

Changed:
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
#[serial]
fn print_no_diff() {
    cmd_with_rustdoc_json_args(
        &[
            "./tests/crates/example_api-v0.2.0",
            "./tests/crates/example_api-v0.2.0",
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

#[test]
fn short_help() {
    let mut cmd = Command::cargo_bin("public_items").unwrap();
    cmd.arg("-h");
    cmd.assert()
        .stdout(expected_help_text())
        .stderr("")
        .success();
}

#[test]
fn long_help() {
    let mut cmd = Command::cargo_bin("public_items").unwrap();
    cmd.arg("--help");
    cmd.assert()
        .stdout(expected_help_text())
        .stderr("")
        .success();
}

#[test]
fn no_args_shows_help() {
    let mut cmd = Command::cargo_bin("public_items").unwrap();
    cmd.assert()
        .stdout(expected_help_text())
        .stderr("")
        .success();
}

#[test]
fn too_many_args_shows_help() {
    let mut cmd = Command::cargo_bin("public_items").unwrap();
    cmd.args(&["too", "many", "args"]);
    cmd.assert()
        .stdout(expected_help_text())
        .stderr("")
        .success();
}

fn expected_help_text() -> String {
    format!(
        "public_items v{}

Requires at least {}.

NOTE: See https://github.com/Enselic/cargo-public-api for a convenient cargo
wrapper around this program (or to be precise; library) that does everything
automatically.

If you insist of using this low-level utility and thin wrapper, you run it like this:

    public_items <RUSTDOC_JSON_FILE>

where RUSTDOC_JSON_FILE is the path to the output of

    RUSTDOCFLAGS='-Z unstable-options --output-format json' cargo +nightly doc --lib --no-deps

which you can find in

    ./target/doc/${{CRATE}}.json

To diff the public API between two commits, you generate one rustdoc JSON file for each
commit and then pass the path of both files to this utility:

    public_items <RUSTDOC_JSON_FILE_OLD> <RUSTDOC_JSON_FILE_NEW>

To include blanket implementations, pass --with-blanket-implementations.

",
        env!("CARGO_PKG_VERSION"),
        MINIMUM_RUSTDOC_JSON_VERSION,
    )
}

/// Helper to setup a `public_items` [`Command`] with rustdoc JSON path args
/// corresponding to the given crates. Use `final_steps` to specify the
/// remaining steps in the test.
fn cmd_with_rustdoc_json_args(crates: &[&str], final_steps: impl FnOnce(Command)) {
    let mut cmd = Command::cargo_bin("public_items").unwrap();

    for crate_ in crates {
        cmd.arg(rustdoc_json_path_for_crate(crate_));
    }

    final_steps(cmd);
}
