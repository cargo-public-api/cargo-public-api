#!/usr/bin/env bash
set -o nounset -o pipefail -o errexit -o xtrace

# Helpers
pyjq() {
    python3 -c "import json, sys; print(json.load(sys.stdin)${1})"
}

# First, deploy
cargo publish

# If that was successful, push a git tag that matches Cargo.toml version
version=$(cargo read-manifest | pyjq '["version"]')
version_tag="v${version}"
git tag "${version_tag}"
git push origin "${version_tag}"

# Done!
