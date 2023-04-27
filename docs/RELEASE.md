## How to release

### `public-api` and `cargo-public-api`

1. First release `rustdoc-json` and `rustup-toolchain` if needed. See below.
1. Update `public-api/CHANGELOG.md`
1. Create a PR that targets `main` that
    1. Bumps to the same `version` in
        * **Cargo.toml** `[workspace.package.version]`
        * Dependents that you find with `git grep -A1 'path = "../public-api"'`
    2. If you bump 0.x.0 version, also update the [compatibility matrix](https://github.com/Enselic/cargo-public-api#compatibility-matrix).
    1. If `MINIMUM_NIGHTLY_RUST_VERSION` must be bumped, bump it. If you bump it, also
        *  bump it in [compatibility matrix](https://github.com/Enselic/cargo-public-api#compatibility-matrix)
        *  bump it in `cargo-public-api` [installation instructions](https://github.com/Enselic/cargo-public-api#installation)
        * `rm cargo-public-api/MINIMUM_NIGHTLY_VERSION_FOR_TESTS`

1. Preview what the auto-generated release-notes will look like by going [here](https://github.com/cargo-public-api/cargo-public-api.github.io/blob/main/release-notes-preview.md). It is automatically updated, but you can also trigger manually [here](https://github.com/Enselic/cargo-public-api/actions/workflows/Preview-release-notes.yml)
    * The target audience for the release notes is users of the `cargo-public-api` CLI. But we also want to credit contributors to our libraries with a mention in the release notes, so we should also include such PRs in the release notes. For libs we also want to update the corresponding `CHANGELOG.md` file though.
1. For each PR included in the release:
    1. Label with `[category-exclude]` if it shall not be mentioned in the release notes.
    1. Label with `[category-enhancement]` if it shall be in the "New Features" section in the auto-generated release notes.
    1. Label with `[category-bugfix]` if it shall be in the "Bugfixes" section in the auto-generated release notes.
    1. Label with `[category-other]` if it shall be in the "Other Changes" section in the auto-generated release notes.
    1. Label with `[category-public-api]`/`[category-rustdoc-json]`/`[category-rustup-toolchain]` if it shall be in the "`public-api`/`rustdoc-json`/`rustup-toolchain` library" section in the auto-generated release notes.
    1. Tweak the PR title if necessary so it makes up a good release notes entry
1. Wait for CR of the PR created in step 2.
1. Once reviewed and merged, run https://github.com/Enselic/cargo-public-api/actions/workflows/Release-cargo-public-api.yml workflow from `main` ([instructions](https://github.com/Enselic/cargo-public-api/blob/main/docs/development.md#how-to-trigger-main-branch-workflow))
1. Done!

### `rustdoc-json`

1. Run `tag=$(git tag --sort=-creatordate | grep ^rustdoc-json-v | head -n1) ; echo "Latest release is $tag" ; git diff $tag -- rustdoc-json/` to see if a release is needed.
1. Update `rustdoc-json/CHANGELOG.md`
1. Bump the `version` in **rustdoc-json/Cargo.toml**.
    * Also bump dependents: `git grep -A1 'path = "../rustdoc-json"'`
1. Run https://github.com/Enselic/cargo-public-api/actions/workflows/Release-rustdoc-json.yml workflow from `main` ([instructions](https://github.com/Enselic/cargo-public-api/blob/main/docs/development.md#how-to-trigger-main-branch-workflow))
1. Done!

### `rustup-toolchain`

1. Run `tag=$(git tag --sort=-creatordate | grep ^rustup-toolchain-v | head -n1) ; echo "Latest release is $tag" ; git diff $tag -- rustup-toolchain/` to see if a release is needed.
1. Update `rustup-toolchain/CHANGELOG.md`
1. Bump the `version` in **rustup-toolchain/Cargo.toml**
    * Also bump dependents: `git grep -A1 'path = "../rustup-toolchain"'`
1. Run https://github.com/Enselic/cargo-public-api/actions/workflows/Release-rustup-toolchain.yml workflow from `main` ([instructions](https://github.com/Enselic/cargo-public-api/blob/main/docs/development.md#how-to-trigger-main-branch-workflow))
1. Done!

## How to trigger main branch workflow

1. Go to https://github.com/Enselic/cargo-public-api/actions and select workflow in the left column
1. Click the **Run workflow ▼** button to the right
1. Make sure the `main` branch is selected
1. Click **Run workflow**
1. Wait for the workflow to complete
