#!/usr/bin/env bash
set -o errexit -o nounset -o pipefail

# Our tests depend on a few toolchains being installed. We support using
# different toolchains to build rustdoc JSON, so we need a few different
# toolchains installed to test that it works.
#
# This script installs these toolchains if they are not already installed.

# The oldest nightly toolchain that we support
minimal_toolchain=$(cargo run -p public-api -- --print-minimum-rustdoc-json-version)
if [ -z "${minimal_toolchain}" ]; then
    echo "FAIL: Could not figure out minimal_toolchain"
    exit 1
fi

# A toolchain that produces rustdoc JSON that we do not understand how to parse.
unusable_toolchain="nightly-2022-06-01"

toolchains_to_install="
    beta
    $minimal_toolchain
    $unusable_toolchain
"

# Check each toolchain and make sure it is installed.
for toolchain in $toolchains_to_install; do
    if ! cargo "+${toolchain}" -v >/dev/null 2>/dev/null; then
        rustup toolchain install --no-self-update --profile=minimal "${toolchain}"
    fi
done
