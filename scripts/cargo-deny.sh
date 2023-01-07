#!/usr/bin/env bash
set -o nounset -o pipefail -o errexit

cargo deny check
