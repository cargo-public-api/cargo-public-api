#/usr/bin/env bash
set -o nounset

# Run like this:
#
#   cd ~/src/some-lib
#   ~/src/cargo-public-api/scripts/multi-diff.sh $(git tag | grep '^v\?[0-9]\+\.[0-9]\+\.[0-9]\+$')

base_version="$1"
shift

for new_version in $@; do
    echo "$base_version -> $new_version"
    cargo public-api --diff-git-checkouts $base_version $new_version 2>/dev/null || echo "(Failed)"
    base_version=$new_version
done
