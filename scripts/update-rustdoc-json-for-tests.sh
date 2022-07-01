#!/usr/bin/env bash
set -o nounset -o pipefail -o errexit -o xtrace

output_dir="./public-api/tests/rustdoc-json"
src_base="$HOME/src"

# These are the only crates for which we version the rustdoc JSON in the repo.
# The rustdoc JSON for other test-crates are built automaticaly on-demand. The
# reason we put some rustdoc JSON in-repo is so that our examples/ can be based
# on existing rustdoc JSON.
crates="
    example_api-v0.1.0
    example_api-v0.2.0
"

for crate in ${crates}; do
    crate_split=(${crate//-/ })
    name=${crate_split[0]} # E.g. `example_api`
    version=${crate_split[1]} # E.g. `v0.1.0`

    crate_dir="./test-apis/${crate}"
    cargo +nightly rustdoc --lib --manifest-path "${crate_dir}/Cargo.toml" -- -Z unstable-options --output-format json
    cp -v "${crate_dir}/target/doc/${name}.json" "${output_dir}/${name}-${version}.json"
done
