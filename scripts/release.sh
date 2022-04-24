#!/usr/bin/env bash
set -o nounset -o pipefail -o errexit -o xtrace

# First, deploy
cargo publish --dry-run

# If that was successful, push a git tag that matches Cargo.toml version
pyjq() {
    python3 -c "import json, sys; print(json.load(sys.stdin)${1})"
}
version=$(cargo read-manifest | pyjq '["version"]')
version_tag="v${version}"
echo git tag "${version_tag}"
echo git push origin "${version_tag}"

# Done!
