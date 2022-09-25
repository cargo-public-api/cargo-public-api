#!/usr/bin/env bash
set -o nounset -o pipefail -o errexit

# `:-+nightly` means "if unset, use +nightly"
toolchain=${RUSTDOC_JSON_OVERRIDDEN_TOOLCHAIN_HACK:-+nightly}

test_git_dir="/tmp/cargo-public-api-test-repo"
rm -rf "${test_git_dir}"
cargo run -p test-utils -- "${test_git_dir}" ./test-apis

build_for="
    comprehensive_api
    comprehensive_api_proc_macro
    example_api-v0.2.0
"

output_for="
    comprehensive_api
    comprehensive_api_proc_macro
"

for crate in $build_for; do
    cargo ${toolchain} rustdoc --lib --manifest-path "./test-apis/${crate}/Cargo.toml" -- -Z unstable-options --output-format json
done

for crate in $output_for; do
    cargo run -p public-api -- "./test-apis/${crate}/target/doc/${crate}.json" > "public-api/tests/expected-output/${crate}.txt"
done

BLESS=1 RUSTDOC_JSON_OVERRIDDEN_TOOLCHAIN_HACK=${toolchain} cargo test --test '*' -p public-api -p cargo-public-api

# BLESS=1 can't be used yet because this is used in test-invocation-variants.sh
RUSTDOC_JSON_OVERRIDDEN_TOOLCHAIN_HACK=${toolchain} cargo run -p cargo-public-api -- \
      --manifest-path "public-api/Cargo.toml" \
      --color=never > \
      "cargo-public-api/tests/expected-output/public_api_list.txt"
