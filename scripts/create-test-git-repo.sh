#!/usr/bin/env bash
set -o nounset -o pipefail -o errexit # -o xtrace

# The cargo-public-api project is meant to be used in git repositories. Since it
# does "destructive" operations (checkout out arbitrary commits to the working
# tree) we can't use the git repo that hosts this file. We need a special git
# repo for testing purposes.
#
# This script creates a git repo for testing purposes from scratch, by turning
# pre-made versions of an example_api into commits and tags.

# The directory where a git repository shall be created.
dest="${1}"

# The dir where this script is found at. Used to find the path to the
# example_apis.
script_dir="${0%/*}"

# Where the example_api versions can be found.
test_apis_dir="${script_dir}/../test-apis"

# Make sure the dest exists
mkdir -p "${dest}"

# we want to use regular git commands, but git should pretend we are in ${dest}
git_cmd="git -C ${dest}"

# First step: git init
${git_cmd} init --initial-branch main "${dest}"

# Needed to prevent errors in CI
${git_cmd} config user.email "cargo-public-api@example.com"
${git_cmd} config user.name "Cargo Public"

# Now go through all directories and create git commits and tags from them
for v in v0.1.0 v0.1.1 v0.2.0 v0.3.0; do
    cp "${test_apis_dir}/example_api-${v}/Cargo.toml" "${dest}/Cargo.toml"

    mkdir -p "${dest}/src"
    cp "${test_apis_dir}/example_api-${v}/src/lib.rs" "${dest}/src/lib.rs"

    ${git_cmd} add .
    ${git_cmd} commit -m "example_api ${v}"
    ${git_cmd} tag "${v}"
done
