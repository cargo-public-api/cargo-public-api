#!/usr/bin/env bash
set -o nounset -o errexit -o pipefail

for workspace in . rustdoc-json rustup-toolchain; do
    echo "Running: $1 $workspace"
    $1 $workspace
done
