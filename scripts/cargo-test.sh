#!/usr/bin/env bash
set -o nounset -o pipefail -o errexit

INSTA_UPDATE=no # See https://docs.rs/insta/latest/insta/#updating-snapshots
UPDATE_SNAPSHOTS=no
if [ "${1:-}" = "--bless" ]; then
    shift
    INSTA_UPDATE=always
    UPDATE_SNAPSHOTS=yes
fi
export INSTA_UPDATE
export UPDATE_SNAPSHOTS

# Put `cargo-public-api` in $PATH so `cargo` finds it and `cargo public-api`
# works, which some tests depend on.
#
# Since `std::env::set_var()` is unsafe in Rust Edition 2024 we don't even
# try to modify `PATH` inside of tests. Instead we make sure that it is set
# appropriately from the start. Since we don't pass `--release` to the below
# `cargo` commands we use `./target/debug` here and not `./target/release`.
PATH="$(pwd)/target/debug:$PATH" cargo test --locked "$@"
