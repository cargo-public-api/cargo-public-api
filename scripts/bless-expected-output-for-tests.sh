#!/usr/bin/env bash
set -o nounset -o pipefail -o errexit

src="tests/rustdoc_json"
dst="tests/expected_output"

source "scripts/test-crates-and-versions.sh"

for crate in ${crates}; do
    cargo run -- "${src}/${crate}.json" > "${dst}/${crate}.txt"
done

cargo run -- --with-blanket-implementations "${src}/public_items-v0.4.0.json" > "${dst}/public_items-v0.4.0-with-blanket-implementations.txt"
