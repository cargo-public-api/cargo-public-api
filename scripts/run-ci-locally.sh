#!/usr/bin/env bash
set -o nounset -o pipefail -o errexit -o xtrace

# This script tries to emulate a run of CI.yml. If you can run this script
# without errors you can be reasonably sure that CI will pass for real when you
# push the code.

# We set this in GitHub workflow files so we should also set it here.
export CARGO_TERM_COLOR=always

cargo fmt -- --check

RUSTDOCFLAGS='--deny warnings' cargo doc --locked --no-deps --document-private-items

scripts/cargo-clippy.sh

cargo build --locked

cargo test --locked

scripts/cargo-test-without-rustup.sh

if command -v cargo-audit >/dev/null; then
    scripts/cargo-audit.sh
else
    echo "INFO: Not running \`cargo audit\` because it is not installed"
fi

if command -v cargo-deny >/dev/null; then
    scripts/cargo-deny.sh
else
    echo "INFO: Not running \`cargo deny\` because it is not installed"
fi
