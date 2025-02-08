#!/usr/bin/env bash
set -o nounset -o errexit -o pipefail

source ./scripts/utils.sh
workspace_dir="$(workspace_dir_from_name ${1:-})"

case "$workspace_dir" in
.)
    # Put `cargo-public-api` in $PATH so `cargo` finds it and `cargo public-api`
    # works.
    #
    # Since `std::env::set_var()` is unsafe in Rust Edition 2024 we don't even
    # try to modify `PATH` inside of tests. Instead we make sure that it is set
    # appropriately from the start. Since we don't pass `--release` to the below
    # `cargo` commands we use `./target/debug` here and not `./target/release`.
    adjusted_path="$(pwd)/target/debug:$PATH"
    ;;
rustdoc-json)
    adjusted_path="$PATH"
    ;;
rustup-toolchain)
    adjusted_path="$PATH"
    ;;
*)
    echo "ERROR: Unknown workspace_dir: $workspace_dir" >&2
    exit 1
    ;;
esac

cd $workspace_dir

PATH="$adjusted_path" cargo test --locked
