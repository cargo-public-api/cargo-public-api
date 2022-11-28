#!/usr/bin/env bash
set -o nounset -o pipefail -o errexit

./scripts/bless-expected-output-for-tests.sh

./scripts/bless-public-apis.sh
