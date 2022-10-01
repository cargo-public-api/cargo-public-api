#!/usr/bin/env bash
set -o nounset -o pipefail -o errexit

# `:-nightly` means "if unset, use nightly"
toolchain=${RUSTDOC_JSON_OVERRIDDEN_TOOLCHAIN_HACK:-nightly}

BLESS=1 RUSTDOC_JSON_OVERRIDDEN_TOOLCHAIN_HACK=${toolchain} cargo test --test '*' -p public-api -p cargo-public-api
