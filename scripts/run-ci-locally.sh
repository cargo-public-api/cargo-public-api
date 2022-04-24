#!/usr/bin/env bash
set -o errexit -o nounset -o pipefail -o xtrace

cargo fmt -- --check

RUSTDOCFLAGS='-D warnings' cargo doc --locked --no-deps

cargo clippy --locked --all-targets --all-features -- -D clippy::all -D clippy::pedantic

cargo test --locked

./scripts/test-invocation-variants.sh
