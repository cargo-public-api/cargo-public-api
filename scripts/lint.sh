#!/usr/bin/env bash
set -o nounset -o errexit -o pipefail

cargo fmt -- --check

RUSTDOCFLAGS='--deny warnings' cargo doc --locked --no-deps --document-private-items

cargo clippy \
    --locked \
    --all-targets \
    --all-features \
    -- \
    --deny clippy::all \
    --deny warnings \
    --forbid unsafe_code

# Only --deny missing_docs for our libs because it does not matter for bins
cargo clippy \
    --locked \
    --lib \
    --package public-api \
    --package rustdoc-json \
    --package rustup-toolchain \
    --all-features \
    -- \
    --deny missing_docs

source ./scripts/utils.sh

if if_command_exists_or_in_ci cargo-audit; then
    cargo audit --deny warnings
fi

if if_command_exists_or_in_ci cargo-deny; then
    cargo deny check
fi

if if_command_exists_or_in_ci shfmt; then
    ./scripts/shfmt.sh --diff
fi
