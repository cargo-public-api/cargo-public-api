#!/usr/bin/env bash
set -o nounset -o pipefail -o errexit

# `:-+nightly` means "if unset, use +nightly"
toolchain=${RUSTDOC_JSON_OVERRIDDEN_TOOLCHAIN_HACK:-+nightly}

test_git_dir="/tmp/cargo-public-api-test-repo"
[ -d "${test_git_dir}" ] || ./scripts/create-test-git-repo.sh "${test_git_dir}"

for crate in comprehensive_api comprehensive_api_proc_macro; do
    cargo ${toolchain} rustdoc --lib --manifest-path "./test-apis/${crate}/Cargo.toml" -- -Z unstable-options --output-format json
    cargo run -p public-api -- "./test-apis/${crate}/target/doc/${crate}.json" > "public-api/tests/expected-output/${crate}.txt"
done

RUSTDOC_JSON_OVERRIDDEN_TOOLCHAIN_HACK=${toolchain} cargo run -p public-api -- --with-blanket-implementations "./public-api/tests/rustdoc-json/example_api-v0.2.0.json" > \
      "public-api/tests/expected-output/example_api-v0.2.0-with-blanket-implementations.txt"

RUSTDOC_JSON_OVERRIDDEN_TOOLCHAIN_HACK=${toolchain} cargo run -p cargo-public-api -- \
      --manifest-path "${test_git_dir}/Cargo.toml" \
      --color=never --diff-git-checkouts "v0.2.0" "v0.3.0" > \
      "cargo-public-api/tests/expected-output/example_api_diff_v0.2.0_to_v0.3.0.txt"

RUSTDOC_JSON_OVERRIDDEN_TOOLCHAIN_HACK=${toolchain} cargo run -p cargo-public-api -- \
      --manifest-path "${test_git_dir}/Cargo.toml" \
      --color=never > \
      "cargo-public-api/tests/expected-output/test_repo_api_latest.txt"

RUSTDOC_JSON_OVERRIDDEN_TOOLCHAIN_HACK=${toolchain} cargo run -p cargo-public-api -- \
      --manifest-path "${test_git_dir}/Cargo.toml" \
      --color=always --diff-git-checkouts "v0.1.0" "v0.2.0" > \
      "cargo-public-api/tests/expected-output/example_api_diff_v0.1.0_to_v0.2.0_colored.txt"

RUSTDOC_JSON_OVERRIDDEN_TOOLCHAIN_HACK=${toolchain} cargo run -p cargo-public-api -- \
      --manifest-path "cargo-public-api/Cargo.toml" \
      --color=always > \
      "cargo-public-api/tests/expected-output/list_self_test_lib_items_colored.txt"
