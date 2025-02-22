#!/usr/bin/env bash
set -o nounset -o errexit -o pipefail

nightly_version=nightly-$(date +'%Y-%m-%d')

echo $nightly_version > cargo-public-api/MINIMUM_NIGHTLY_RUST_VERSION_FOR_TESTS

git add .

git commit \
  --author "EnselicCICD <junta-pixlar0l@icloud.com>" \
  --message "Bless \`nightly_version\` output

Automatically created by $CURRENT_JOB_URL
"

git branch auto-bless/$nightly_version



git push 
          push-to-fork: EnselicCICD/cargo-public-api
          branch: create-pull-request/${{ steps.latest-nightly.outputs.version }}
