#!/usr/bin/env bash
set -o nounset -o pipefail -o errexit

zizmor \
    --config zizmor.yml \
    .github/workflows/* \
    "$@"
