#!/usr/bin/env bash
set -o nounset -o errexit -o pipefail

# When this script is run there are already local git repo changes with new
# blessed output using latest nightly toolchain. Create a branch to work with.
nightly_version="nightly-$(date +'%Y-%m-%d')"
branch_name="auto-bless/$nightly_version"
git checkout \
    -b "$branch_name"

# Update MINIMUM_NIGHTLY_RUST_VERSION_FOR_TESTS and then commit all changes.
git config user.name "Martin Nordholts CI/CD"
git config user.email "104096785+EnselicCICD@users.noreply.github.com"
echo $nightly_version >cargo-public-api/MINIMUM_NIGHTLY_RUST_VERSION_FOR_TESTS
git add .
git commit --message "Bless \`$nightly_version\` output

Automatically created by $CURRENT_JOB_URL
"

# Create the PR
git push https://EnselicCICD@github.com/EnselicCICD/cargo-public-api "$branch_name"
gh config set prompt disabled
gh pr create \
    --repo cargo-public-api/cargo-public-api \
    --base main \
    --head EnselicCICD:"$branch_name" \
    --fill
