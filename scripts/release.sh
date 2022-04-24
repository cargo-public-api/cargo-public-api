#!/usr/bin/env bash
set -o nounset -o pipefail -o errexit -o xtrace

# First, deploy
cargo publish

# If that was successful, push a git tag that matches Cargo.toml version
pyjq() {
    python3 -c "import json, sys; print(json.load(sys.stdin)${1})"
}
version=$(cargo read-manifest | pyjq '["version"]')
version_tag="v${version}"
version_tag="temp-tag-test"
git tag "${version_tag}"
git push origin "${version_tag}"

# Done!
