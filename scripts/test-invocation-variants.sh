#!/usr/bin/env bash
set -o errexit -o nounset -o pipefail

# The script assumes that cargo-public-api is the current dir
cd cargo-public-api

td=$(mktemp -d)

# We expect this public API to be printed to stdout
expected_stdout="pub fn cargo_public_api::for_self_testing_purposes_please_ignore()
pub mod cargo_public_api
pub use cargo_public_api::public_api"

# We want the tool to print progress when it builds rustdoc JSON. The presence
# of this string is what we use to test if that is the case.
expected_stderr="Documenting cargo-public-api"

# We write stdout and stderr to temporary files so we can later assert on their
# content.
stdout_path="${td}/cargo-public-api-test-invocation-variants-stdout"
stderr_path="${td}/cargo-public-api-test-invocation-variants-stderr"

# Clean the temp directory
trap 'rm -rf "${td}"' EXIT

# Helper that runs the command passed as the first arg, and then asserts on
# stdout and stderr
assert_progress_and_output() {
    local cmd="$1"

    echo -n "${cmd} ... "

    CARGO_TERM_COLOR=never ${cmd} >${stdout_path} 2>${stderr_path}

    local actual_stdout=$(cat ${stdout_path})
    local actual_stderr=$(cat ${stderr_path})

    rm ${stdout_path} ${stderr_path}

    if [[ ${actual_stdout} != ${expected_stdout} ]]; then
        echo -e "FAIL: \n${actual_stdout}\n!=\n${expected_stdout}"
        exit 1
    fi

    if [[ "${actual_stderr}" != *"${expected_stderr}"* ]]; then
        echo -e "FAIL: \n${actual_stderr}\ndoes not contain \`${expected_stderr}\`"
        exit 1
    fi

    echo "PASS"
}

# Now we are ready to run the actual tests

# Make sure we can conveniently run the tool from the source dir
assert_progress_and_output "cargo run"

# Make sure we can conveniently run the tool from the source dir on an external crate
assert_progress_and_output "cargo run -- --manifest-path $(pwd)/Cargo.toml"

# Install the tool
cargo install --debug --path .

# Make sure we can run the tool on the current directory stand-alone
assert_progress_and_output "cargo-public-api"

# Make sure we can run the tool on an external directory stand-alone
assert_progress_and_output "cargo-public-api --manifest-path $(pwd)/Cargo.toml"

# Make sure we can run the tool with a specified package from a virtual manifest
(cd .. && assert_progress_and_output "cargo-public-api --package cargo-public-api")

# Make sure we can run the tool on the current directory as a cargo sub-command
assert_progress_and_output "cargo public-api"

# Make sure we can run the tool on an external directory as a cargo sub-command
assert_progress_and_output "cargo public-api --manifest-path $(pwd)/Cargo.toml"

# Make sure we can run the tool with MINIMUM_RUSTDOC_JSON_VERSION
rustup toolchain install nightly-2022-08-15
assert_progress_and_output "cargo +nightly-2022-08-15 public-api --manifest-path $(pwd)/Cargo.toml"
