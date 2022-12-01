#!/usr/bin/env bash
set -o nounset -o pipefail -o errexit

crates="
    public-api
    rustdoc-json
    rustup-toolchain
"
for crate in $crates; do
    cargo run -p cargo-public-api -- --manifest-path ${crate}/Cargo.toml > ${crate}/public-api.txt
done
