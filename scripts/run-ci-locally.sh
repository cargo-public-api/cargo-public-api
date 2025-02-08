#!/usr/bin/env bash
set -o nounset -o errexit -o pipefail

# This script tries to emulate a run of CI.yml. If you can run this script
# without errors you can be reasonably sure that CI will pass for real when you
# push the code.

# We set this in GitHub workflow files so we should also set it here.
export CARGO_TERM_COLOR=always

./scripts/for-each-workspace.sh ./scripts/lint-workspace.sh
./scripts/for-each-workspace.sh ./scripts/test-workspace.sh
./scripts/cargo-test-without-rustup.sh
