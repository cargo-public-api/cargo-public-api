#!/usr/bin/env bash
set -o nounset -o pipefail -o errexit

base="tests/rustdoc_json"

crates="
    bat-v0.19.0
    syntect-v4.6.0
    thiserror-v1.0.30
"

for crate in ${crates}; do
    # Note that we must remove the trailing newline, otherwise `.split('\n')`
    # will yield an empty last item
    printf "%s" "$(cargo run ${base}/${crate}_FORMAT_VERSION_10.json)" > "${base}/${crate}-expected.txt"
done
