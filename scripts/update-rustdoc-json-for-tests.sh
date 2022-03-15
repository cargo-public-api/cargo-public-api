#!/usr/bin/env bash
set -o nounset -o pipefail -o errexit -o xtrace

if [[ `git status --porcelain` ]]; then
  echo "Abording. This script does git checkout --force in public_items repo. Make sure to commit your local changes first!"
  exit 1
fi

output_dir=$(mktemp -d)
src_base="$HOME/src"

source "scripts/test-crates-and-versions.sh"

for crate in ${crates}; do
    crate_split=(${crate//-/ })
    name=${crate_split[0]} # E.g. `bat`
    tag=${crate_split[1]} # E.g. `v0.19.0`

    cd "${src_base}/${name}"

    if [ "public_items" = "${name}" ]; then
        git checkout --force "${tag}"
    else
        git checkout "${tag}"
    fi

    RUSTDOCFLAGS='-Z unstable-options --output-format json' cargo +nightly doc --lib --no-deps
    cp -v ./target/doc/${name}.json ${output_dir}/${name}-${tag}.json
done

echo "New JSON put in ${output_dir}"
