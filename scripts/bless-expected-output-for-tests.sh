#!/usr/bin/env bash
set -o nounset -o pipefail -o errexit

src="tests/rustdoc_json"
dst="tests/expected_output"

source "scripts/test-crates-and-versions.sh"

for crate in ${crates}; do
    cargo run -- "${src}/${crate}.json" > "${dst}/${crate}.txt"
done

cargo run -- --with-blanket-implementations "${src}/public_items-v0.4.0.json" > "${dst}/public_items-v0.4.0-with-blanket-implementations.txt"

RUSTDOCFLAGS='-Z unstable-options --output-format json' cargo +nightly doc --manifest-path ./tests/crates/comprehensive_api/Cargo.toml --lib --no-deps
cargo run ./target/doc/comprehensive_api.json > "${dst}/comprehensive_api.txt"
