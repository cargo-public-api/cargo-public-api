#!/usr/bin/env bash
set -o errexit -o nounset -o pipefail -o xtrace

# Should show --help
cargo run
cargo run -- --help
cargo run -- -h
cargo run -- too many args

# Should show public items
cargo run -- ./tests/rustdoc_json/public_items-v0.4.0.json

# Should show diff of public items
cargo run -- ./tests/rustdoc_json/public_items-v0.2.0.json ./tests/rustdoc_json/public_items-v0.4.0.json
