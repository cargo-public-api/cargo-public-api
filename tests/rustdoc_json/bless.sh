#!/usr/bin/env bash
set -o nounset -o pipefail -o errexit

base="tests/rustdoc_json"

source "${base}/crates.sh"

for crate in ${crates}; do
    # Note that we must remove the trailing newline, otherwise `.split('\n')`
    # will yield an empty last item
    printf "%s" "$(cargo run -- ${base}/${crate}.json)" > "${base}/${crate}-expected.txt"
done

printf "%s" "$(cargo run -- --with-blanket-implementations ${base}/public_items-git.json)" > "${base}/public_items-git-expected-with-blanket-implementations.txt"
