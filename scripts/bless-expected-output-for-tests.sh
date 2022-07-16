#!/usr/bin/env bash
set -o nounset -o pipefail -o errexit

for crate in comprehensive_api comprehensive_api_proc_macro; do
    cargo +nightly rustdoc --lib --manifest-path "./test-apis/${crate}/Cargo.toml" -- -Z unstable-options --output-format json
    cargo run -p public-api -- "./test-apis/${crate}/target/doc/${crate}.json" > "public-api/tests/expected-output/${crate}.txt"
done

cargo run -p public-api -- --with-blanket-implementations "./public-api/tests/rustdoc-json/example_api-v0.2.0.json" > \
      "public-api/tests/expected-output/example_api-v0.2.0-with-blanket-implementations.txt"

cargo run -p cargo-public-api -- \
      --manifest-path "target/tmp/cargo-public-api-test-repo/Cargo.toml" \
      --color=never --diff-git-checkouts "v0.0.4" "v0.0.5" > \
      "cargo-public-api/tests/expected-output/test_crate_diff_v0.0.4_to_v0.0.5.txt"
