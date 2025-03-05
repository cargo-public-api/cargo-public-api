#!/usr/bin/env bash
set -o nounset -o pipefail -o errexit -o xtrace

# This script tries to emulate a run of CI.yml. If you can run this script
# without errors you can be reasonably sure that CI will pass for real when you
# push the code.

# We set this in GitHub workflow files so we should also set it here.
export CARGO_TERM_COLOR=always

./scripts/lint.sh

./scripts/cargo-test.sh
