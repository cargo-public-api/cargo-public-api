#!/usr/bin/env bash
set -o nounset -o pipefail -o errexit

base="tests/rustdoc_json"

source "${base}/crates.sh"

for crate in ${crates}; do
    # Note that we must remove the trailing newline, otherwise `.split('\n')`
    # will yield an empty last item

    echo "${crate}"
    if [ "public_items-git" = "${crate}" ]; then
        printf "%s" "$(cargo run -- --omit-blanket-implementations ${base}/${crate}.json)" > "${base}/${crate}-expected.txt"
    else
        printf "%s" "$(cargo run -- ${base}/${crate}.json)" > "${base}/${crate}-expected.txt"
    fi
done
