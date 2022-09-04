#!/usr/bin/env bash
set -o errexit -o nounset -o pipefail

# The oldest nightly toolchain that we support
minimal_toolchain=$(sed -n 's/pub const MINIMUM_RUSTDOC_JSON_VERSION: &str = "\(nightly-[^"]\+\)";/\1/p' public-api/src/lib.rs)

# A toolchain that produces rustdoc JSON that we do not understand how to parse.
unusable_toolchain="nightly-2022-06-01"

toolchains_to_install="
    beta
    $minimal_toolchain
    $unusable_toolchain
"

# These tests depends on some specific toolchains being installed. Install them
# if they are not already installed.
for toolchain in $toolchains_to_install; do
    if ! cargo "+${toolchain}" -v >/dev/null 2>/dev/null; then
        rustup toolchain install --no-self-update "${toolchain}"
    fi
done

# The script assumes that cargo-public-api is the current dir
cd cargo-public-api

td=$(mktemp -d)

# We write stdout and stderr to temporary files so we can later assert on their
# content.
stdout_path="${td}/cargo-public-api-test-invocation-variants-stdout"
stderr_path="${td}/cargo-public-api-test-invocation-variants-stderr"

# Clean the temp directory on EXIT so the tests don't have to worry about cleanup
trap 'rm -rf "${td}"' EXIT

# Helper that runs the command passed as the first arg, and then asserts on
# stdout and stderr
assert_progress_and_output() {
    local cmd="$1"
    local expected_stdout_path="$2"
    local expected_stderr_substring="$3"

    echo -n "${cmd} ... "

    CARGO_TERM_COLOR=never ${cmd} >${stdout_path} 2>${stderr_path}

    local actual_stderr=$(cat ${stderr_path})

    if ! diff -u "${expected_stdout_path}" "${stdout_path}"; then
        echo -e "FAIL: \`diff -u ${expected_stdout_path} ${stdout_path}\` was not empty"
        exit 1
    fi

    if [[ "${actual_stderr}" != *"${expected_stderr_substring}"* ]]; then
        echo -e "FAIL: \n${actual_stderr}\ndoes not contain \`${expected_stderr_substring}\`"
        exit 1
    fi

    echo "PASS"
}

# Now we are ready to run the actual tests

# Make sure we can conveniently run the tool from the source dir
# We want the tool to print progress when it builds rustdoc JSON. The presence
# of "Documenting cargo-public-api" on stderr is what we use to test if that is
# the case.
assert_progress_and_output \
    "cargo run" \
    tests/expected-output/list_self_test_lib_items.txt \
    "Documenting cargo-public-api"

# Make sure we can conveniently run the tool from the source dir on an external crate
assert_progress_and_output \
    "cargo run -- --manifest-path $(pwd)/Cargo.toml" \
    tests/expected-output/list_self_test_lib_items.txt \
    "Documenting cargo-public-api"

# Install the tool
cargo install --debug --path .

# Make sure we can run the tool on the current directory stand-alone
assert_progress_and_output \
    "cargo-public-api" \
    tests/expected-output/list_self_test_lib_items.txt \
    "Documenting cargo-public-api"

# Make sure we can run the tool on an external directory stand-alone
assert_progress_and_output \
    "cargo-public-api --manifest-path $(pwd)/Cargo.toml" \
    tests/expected-output/list_self_test_lib_items.txt \
    "Documenting cargo-public-api"

# Make sure we can run the tool with a specified package from a virtual manifest
(cd .. && assert_progress_and_output \
    "cargo-public-api --package cargo-public-api" \
    cargo-public-api/tests/expected-output/list_self_test_lib_items.txt \
    "Documenting cargo-public-api")

# Make sure we can run the tool on the current directory as a cargo sub-command
assert_progress_and_output \
    "cargo public-api" \
    tests/expected-output/list_self_test_lib_items.txt \
    "Documenting cargo-public-api"

# Make sure we can run the tool on an external directory as a cargo sub-command
assert_progress_and_output \
    "cargo public-api --manifest-path $(pwd)/Cargo.toml" \
    tests/expected-output/list_self_test_lib_items.txt \
    "Documenting cargo-public-api"

# Make sure cargo subcommand args filtering of 'public-api' is not too aggressive
assert_progress_and_output \
    "cargo public-api -p public-api" \
    tests/expected-output/public_api_list.txt \
    "Documenting public-api"

# Make sure we can run the tool with MINIMUM_RUSTDOC_JSON_VERSION
assert_progress_and_output \
    "cargo +nightly-2022-08-15 public-api --manifest-path $(pwd)/Cargo.toml" \
    tests/expected-output/list_self_test_lib_items.txt \
    "Documenting cargo-public-api"

# Sanity check to make sure we can make the tool build rustdoc JSON with a
# custom toolchain via the rustup proxy mechanism (see
# https://rust-lang.github.io/rustup/concepts/index.html#how-rustup-works). The
# test uses a too old nightly toolchain, which should make the tool fail if it's used.
cmd="cargo public-api --toolchain +${unusable_toolchain}"
echo -n "${cmd} ... "
if ${cmd} >/dev/null 2>/dev/null; then
    echo "FAIL: Using '${unusable_toolchain}' to build rustdoc JSON should have failed!"
    exit 1
else
    echo "PASS"
fi

cmd="cargo +${unusable_toolchain} public-api"
echo -n "${cmd} ... "
if ${cmd} >/dev/null 2>/dev/null; then
    echo "FAIL: Using '${unusable_toolchain}' to build rustdoc JSON should have failed!"
    exit 1
else
    echo "PASS"
fi


cmd="cargo +beta public-api"
echo -n "${cmd} ... "
if n=$(${cmd} 2>&1 | grep "Warning: using the \`beta.*\` toolchain for gathering the public api is not possible"); then
    echo "PASS"
else
    echo "FAIL: Using '+beta' to build rustdoc JSON should have mentioned the upgrade to `+nightly`"
    exit 1
fi
