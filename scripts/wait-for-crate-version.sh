#/usr/bin/env bash
set -o nounset -o errexit -o xtrace

crate="$1"
version="$2"

attempts_left="10"
while [ "$attempts_left" != "0" ]; do
    sleep 15
    actual_version=$(curl -s -L "https://crates.io/api/v1/crates/$crate/$version" | jq --raw-output .version.num)
    if [ "$actual_version" = "$version" ]; then
        break
    fi

    ((attempts_left=attempts_left-1))
done

if [ "$attempts_left" = "0" ]; then
    echo "Version $version of crate $crate never appeared :("
fi
