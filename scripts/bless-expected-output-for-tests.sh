#!/usr/bin/env bash
set -o nounset -o pipefail -o errexit

# `:-+nightly` means "if unset, use +nightly"
toolchain=${RUSTDOC_JSON_OVERRIDDEN_TOOLCHAIN_HACK:-+nightly}

for crate in comprehensive_api comprehensive_api_proc_macro; do
    cargo ${toolchain} rustdoc --lib --manifest-path "./test-apis/${crate}/Cargo.toml" -- -Z unstable-options --output-format json
    cargo run -p public-api -- "./test-apis/${crate}/target/doc/${crate}.json" > "public-api/tests/expected-output/${crate}.txt"
done

cargo run -p public-api -- --with-blanket-implementations "./public-api/tests/rustdoc-json/example_api-v0.2.0.json" > \
      "public-api/tests/expected-output/example_api-v0.2.0-with-blanket-implementations.txt"

cargo run -p cargo-public-api -- \
      --manifest-path "target/tmp/cargo-public-api-test-repo/Cargo.toml" \
      --color=never --diff-git-checkouts "v0.2.0" "v0.3.0" > \
      "cargo-public-api/tests/expected-output/example_api_diff_v0.2.0_to_v0.3.0.txt"

cargo run -p cargo-public-api -- \
      --manifest-path "target/tmp/cargo-public-api-test-repo/Cargo.toml" \
      --color=always --diff-git-checkouts "v0.1.0" "v0.2.0" > \
      "cargo-public-api/tests/expected-output/example_api_diff_v0.1.0_to_v0.2.0_colored.txt"

cargo run -p cargo-public-api -- \
      --manifest-path "cargo-public-api/Cargo.toml" \
      --color=always > \
      "cargo-public-api/tests/expected-output/list_self_test_lib_items_colored.txt"
