#!/usr/bin/env bash
set -o nounset -o pipefail -o errexit

UPDATE_EXPECT=1 ./scripts/cargo-test.sh
