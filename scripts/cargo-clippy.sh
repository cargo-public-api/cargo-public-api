#!/usr/bin/env bash
set -o nounset -o pipefail -o errexit

# You can run this script from within your IDE (see `.vscode/tasks.json` for an
# example configuration)

# Applies to all packages
cargo clippy \
    --locked \
    --all-targets \
    --all-features \
    -- \
    --deny clippy::all \
    --deny clippy::pedantic \
    --deny warnings \
    --deny unsafe_code \

# Only --deny missing_docs for our libs because it does not matter for bins
cargo clippy \
    --locked \
    --lib \
    --package rustdoc-json \
    --package public-api \
    --all-features \
    -- \
    --deny missing_docs \
