#!/usr/bin/env bash
set -o nounset -o pipefail

./scripts/bless-expected-output-for-tests.sh

./scripts/bless-public-apis.sh
