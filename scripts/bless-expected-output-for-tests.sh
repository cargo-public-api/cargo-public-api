#!/usr/bin/env bash
set -o nounset -o pipefail -o errexit

BLESS=1 cargo test --test '*' -p public-api -p cargo-public-api
