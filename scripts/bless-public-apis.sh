#!/usr/bin/env bash
set -o nounset -o pipefail -o errexit

for crate in public-api rustdoc-json; do
    cargo run -p cargo-public-api -- --manifest-path ${crate}/Cargo.toml > ${crate}/public-api.txt
done
