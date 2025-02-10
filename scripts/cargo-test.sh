#!/usr/bin/env bash
set -o nounset -o pipefail -o errexit

# Put `cargo-public-api` in $PATH so `cargo` finds it and `cargo public-api`
# works.
#
# Since `std::env::set_var()` is unsafe in Rust Edition 2024 we don't even
# try to modify `PATH` inside of tests. Instead we make sure that it is set
# appropriately from the start. Since we don't pass `--release` to the below
# `cargo` commands we use `./target/debug` here and not `./target/release`.
PATH="$(pwd)/target/debug:$PATH" cargo test --locked
