#!/usr/bin/env bash
set -o nounset -o errexit -o pipefail

source ./scripts/utils.sh
workspace_dir="$(workspace_dir_from_name ${1:-})"

case "$workspace_dir" in
.)
    library_package="public-api"
    run_shfmt=true # We only need to run `shfmt` from the repo root
    ;;
rustdoc-json)
    library_package="rustdoc-json"
    run_shfmt=false
    ;;
rustup-toolchain)
    library_package="rustup-toolchain"
    run_shfmt=false
    ;;
*)
    echo "ERROR: Unknown workspace_dir: $workspace_dir" >&2
    exit 1
    ;;
esac

if $run_shfmt && if_command_exists_or_in_ci shfmt; then
    ./scripts/shfmt.sh --diff
fi

cd "$workspace_dir"

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
    --package $library_package \
    --all-features \
    -- \
    --deny missing_docs

if if_command_exists_or_in_ci cargo-audit; then
    cargo audit --deny warnings
fi

if if_command_exists_or_in_ci cargo-deny; then
    cargo deny check
fi
