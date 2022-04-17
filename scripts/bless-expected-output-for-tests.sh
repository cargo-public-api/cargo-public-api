#!/usr/bin/env bash
set -o nounset -o pipefail -o errexit

dst="tests/expected-output"

crates="
    comprehensive_api
    comprehensive_api_proc_macro
"

for crate in ${crates}; do
    RUSTDOCFLAGS='-Z unstable-options --output-format json' cargo +nightly doc \
            --manifest-path "./tests/crates/${crate}/Cargo.toml" --lib --no-deps
    cargo run "./tests/crates/${crate}/target/doc/${crate}.json" > "${dst}/${crate}.txt"
done

cargo run -- --with-blanket-implementations "./tests/rustdoc-json/example_api-v0.2.0.json" > \
      "${dst}/example_api-v0.2.0-with-blanket-implementations.txt"
