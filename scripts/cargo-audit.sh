#!/usr/bin/env bash
set -o nounset -o pipefail -o errexit

cargo audit --deny warnings
