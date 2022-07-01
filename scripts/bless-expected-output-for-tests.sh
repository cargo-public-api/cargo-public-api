#!/usr/bin/env bash
set -o nounset -o pipefail -o errexit

dst="public-api/tests/expected-output"

crates="
    comprehensive_api
    comprehensive_api_proc_macro
"

for crate in ${crates}; do
    cargo +nightly rustdoc --lib --manifest-path "./test-apis/${crate}/Cargo.toml" -- -Z unstable-options --output-format json
    cargo run -p public-api -- "./test-apis/${crate}/target/doc/${crate}.json" > "${dst}/${crate}.txt"
done

cargo run -p public-api -- --with-blanket-implementations "./public-api/tests/rustdoc-json/example_api-v0.2.0.json" > \
      "${dst}/example_api-v0.2.0-with-blanket-implementations.txt"
