#!/usr/bin/env bash
set -o errexit -o nounset -o pipefail -o xtrace

cd cargo-public-api

# Make sure we can conveniently run the tool from the source dir
cargo run

# Make sure we can conveniently run the tool from the source dir on an external crate
cargo run -- --manifest-path "$(pwd)"/Cargo.toml

# Install the tool
cargo install --debug --path .

# Make sure we can run the tool on the current directory stand-alone
cargo-public-api

# Make sure we can run the tool on an external directory stand-alone
cargo-public-api --manifest-path "$(pwd)"/Cargo.toml

# Make sure we can run the tool on the current directory as a cargo sub-command
cargo public-api

# Make sure we can run the tool on an external directory as a cargo sub-command
cargo public-api --manifest-path "$(pwd)"/Cargo.toml
