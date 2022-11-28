#!/usr/bin/env bash
set -o nounset -o pipefail -o errexit

# Bless
./scripts/bless.sh

# If nothing changed there is nothing more to do
if [ -z "$(git status --porcelain)" ]; then
    echo "Nothing to bless so nothing to do, exiting"
    exit 0
fi

# Figure out name of current nightly
current_nightly=$(echo nightly-$(date +%Y-%m-%d))

# Prepare a branch to create a PR from
# With `date +%s` so that repeated runs don't get the same branch name
branch_name="auto-bless-${current_nightly}-$(date +%s)"
git checkout -b "${branch_name}"

# Commit the blessed changes and push the branch
title='Bless `scripts/auto-bless.sh` output'
body="Automatically created by ${GITHUB_SERVER_URL}/${GITHUB_REPOSITORY}/actions/runs/${GITHUB_RUN_ID}

To trigger CI checks, please close and re-open the PR."
git config user.email "junta-pixlar0l@icloud.com"
git config user.name "EnselicCICD"
git commit -a -m "${title}"
git push origin "${branch_name}"

# Create a PR from the newly pushed branch
gh pr create \
    --head "${branch_name}" \
    --title "${title}" \
    --body "${body}" \
    --label "category-exclude" \
