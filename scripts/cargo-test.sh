#!/usr/bin/env bash
set -o nounset -o pipefail -o errexit

# Make it easy to bless changed output. Simply run this to bless:
#
#   ./scripts/cargo-test.sh --bless
if [ "${1:-}" = "--bless" ]; then
    export UPDATE_EXPECT=1
fi

# Since `std::env::set_var()` is unsafe in Rust Edition 2024 we don't even
# try to modify `PATH` inside of tests. Instead we make sure that it is set
# appropriately from the start. Since we don't pass `--release` to the below
# `cargo` commands we use `./target/debug` here and not `./target/release`.
PATH="$(pwd)/target/debug:$PATH" cargo test --locked
