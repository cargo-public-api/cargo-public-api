#!/usr/bin/env bash
set -o nounset -o pipefail -o errexit

base="tests/rustdoc_json"

for input in syntect-v4.6.0 thiserror-v1.0.30; do
    printf "%s" "$(cargo run ${base}/${input}_FORMAT_VERSION_10.json)" > "${base}/${input}-expected.txt"
done
