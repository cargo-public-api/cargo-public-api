#!/usr/bin/env bash
set -o nounset -o pipefail -o errexit

for test_api in ./test-apis/*; do
    echo "Checking ${test_api} ..."

    cargo fmt --check --manifest-path ${test_api}/Cargo.toml

    if [ "${test_api}" != "./test-apis/lint_error" ]; then
        RUSTFLAGS='--deny warnings' cargo check --manifest-path ${test_api}/Cargo.toml
    fi
done
