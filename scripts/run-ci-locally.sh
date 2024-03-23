#!/usr/bin/env bash
set -o nounset -o pipefail -o errexit -o xtrace

# CI sets this, so we should too
export CARGO_TERM_COLOR=always

# This script tries to emulate a run of CI.yml. If you can run this script
# without errors you can be reasonably sure that CI will pass for real when you
# push the code.

cargo fmt -- --check

RUSTDOCFLAGS='--deny warnings' cargo doc --locked --no-deps --document-private-items

scripts/cargo-clippy.sh

cargo build --locked # Build with default features

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
