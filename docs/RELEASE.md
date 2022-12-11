## How to release

### `public-api` and `cargo-public-api`

1. First release `rustdoc-json` and `rustup-toolchain` if needed. See below.
1. Update `public-api/CHANGELOG.md`
1. Create a PR that targets `main` that
    1. Bumps to the same `version` in
        * **Cargo.toml** `[workspace.package.version]`
        * Dependents that you find with `git grep -A1 'path = "../public-api"'`
    2. If you bump 0.x.0 version, also update the [compatibility matrix](https://github.com/Enselic/cargo-public-api#compatibility-matrix).
    1. If `MINIMUM_RUSTDOC_JSON_VERSION` must be bumped, bump it. If you bump it, also bump it in
        * [compatibility matrix](https://github.com/Enselic/cargo-public-api#compatibility-matrix)
        * `cargo-public-api` [installation instructions](https://github.com/Enselic/cargo-public-api#installation)
        * `public-api` [installation instructions](https://github.com/Enselic/cargo-public-api/tree/main/public-api#usage)

1. Preview what the auto-generated release-notes will look like by going [here](https://github.com/cargo-public-api/cargo-public-api.github.io/blob/main/release-notes-preview.md). It is automatically updated, but you can also trigger manually [here](https://github.com/Enselic/cargo-public-api/actions/workflows/Peek-release-notes.yml)
1. For each PR included in the release:
    1. Label with `[category-exclude]` if it shall not be mentioned in the release notes.
    1. Label with `[category-enhancement]` if it shall be in the "New Features" section in the auto-generated release notes.
    1. Label with `[category-bugfix]` if it shall be in the "Bugfixes" section in the auto-generated release notes.
    1. Label with `[category-public_api]` if it shall be in the "`public_api` library" section in the auto-generated release notes.
    1. Tweak the PR title if necessary so it makes up a good release notes entry
1. Wait for CR of the PR created in step 2.
1. Once reviewed and merged, run https://github.com/Enselic/cargo-public-api/actions/workflows/Release.yml workflow from `main` ([instructions](https://github.com/Enselic/cargo-public-api/blob/main/docs/development.md#how-to-trigger-main-branch-workflow))
1. Double-check that the release ended up at https://crates.io/crates/public-api/versions and https://crates.io/crates/cargo-public-api/versions
1. Double-check that the auto-generated release notes for the release at https://github.com/Enselic/cargo-public-api/releases is not horribly inaccurate. If so, please edit.
1. Done!

### `rustdoc-json`

1. Update `rustdoc-json/CHANGELOG.md`
1. Bump the `version` in **rustdoc-json/Cargo.toml** and the dependents that you find with `git grep -A1 'path = "../rustdoc-json"'`.
1. Run https://github.com/Enselic/cargo-public-api/actions/workflows/Release-rustdoc-json.yml workflow from `main` ([instructions](https://github.com/Enselic/cargo-public-api/blob/main/docs/development.md#how-to-trigger-main-branch-workflow))
1. Double-check that the release ended up at https://crates.io/crates/rustdoc-json/versions
1. Done!

### `rustup-toolchain`

1. Update `rustup-toolchain/CHANGELOG.md`
1. Bump the `version` in **rustup-toolchain/Cargo.toml** and the dependency declared in **cargo-public-api/Cargo.toml**.
1. Run https://github.com/Enselic/cargo-public-api/actions/workflows/Release-rustup-toolchain.yml workflow from `main` ([instructions](https://github.com/Enselic/cargo-public-api/blob/main/docs/development.md#how-to-trigger-main-branch-workflow))
1. Double-check that the release ended up at https://crates.io/crates/rustup-toolchain/versions
1. Done!

## How to trigger main branch workflow

1. Go to https://github.com/Enselic/cargo-public-api/actions and select workflow in the left column
1. Click the **Run workflow ▼** button to the right
1. Make sure the `main` branch is selected
1. Click **Run workflow**
1. Wait for the workflow to complete
