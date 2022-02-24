#!/usr/bin/env bash
set -o nounset -o pipefail -o errexit -o xtrace

base="tests/rustdoc_json"
src_base="$HOME/src"

source "${base}/crates.sh"
git rm ${base}/*.json

for crate in ${crates}; do
    crate_split=(${crate//-/ })
    name=${crate_split[0]} # E.g. `bat`
    tag=${crate_split[1]} # E.g. `v0.19.0`

    cd "${src_base}/${name}"
    git checkout "${tag}"
    RUSTDOCFLAGS='-Z unstable-options --output-format json' cargo +nightly doc --lib --no-deps
    cp -v ./target/doc/${name}.json ../public_items/${base}/${name}-${tag}.json
done
