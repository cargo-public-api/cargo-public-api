#!/usr/bin/env bash
set -o nounset -o pipefail -o errexit

if [ "${1:-}" = "--write" ]; then
    MODE="--write"
elif [ "${1:-}" = "--diff" -o -z "${1:-}" ]; then
    MODE="--diff"
fi

if ! shfmt $MODE --indent 4 $(git ls-files | grep '\.sh$'); then
    echo -e "\nERROR: shfmt failed (see above). Try:\n\n    ./scripts/shfmt.sh --write"
    exit 1
fi
