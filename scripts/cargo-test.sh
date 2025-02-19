#!/usr/bin/env bash
set -o nounset -o pipefail -o errexit

# Put `cargo-public-api` in $PATH so `cargo` finds it and `cargo public-api`
# works, which some tests depend on.
#
# Since `std::env::set_var()` is unsafe in Rust Edition 2024 we don't even
# try to modify `PATH` inside of tests. Instead we make sure that it is set
# appropriately from the start. Since we don't pass `--release` to the below
# `cargo` commands we use `./target/debug` here and not `./target/release`.
export PATH="$(pwd)/target/debug:$PATH"

if [ "${1:-}" = "--bless" ]; then
    if ! command -v cargo-insta >/dev/null; then
        echo "ERROR: \`cargo-insta\` not found. Run \`cargo install cargo-insta\` first."
        exit 1
    fi
    cargo insta test || true # avoid errexit if output changed
    cargo insta review
else
    cargo test --locked
fi
