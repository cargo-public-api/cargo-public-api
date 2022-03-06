#!/usr/bin/env bash
set -o errexit -o nounset -o pipefail -o xtrace

cargo fmt -- --check

RUSTDOCFLAGS='-D warnings' cargo doc --no-deps

cargo clippy --all-targets --all-features -- -D clippy::all -D clippy::pedantic

cargo test

./scripts/test-invocation-variants.sh
