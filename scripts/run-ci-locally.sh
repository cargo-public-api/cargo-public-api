#!/usr/bin/env bash
set -o nounset -o pipefail -o errexit -o xtrace

# This script tries to emulate a run of CI.yml. If you can run this script
# without errors you can be reasonably sure that CI will pass for real when you
# upload the code.

# CI.yml
cargo fmt -- --check
RUSTDOCFLAGS='-D warnings' cargo doc  --locked --no-deps
cargo clippy --locked --all-targets --all-features -- -D clippy::all -D clippy::pedantic
cargo test --locked
