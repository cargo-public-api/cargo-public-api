## Versioning strategy

* **x.0.0**: We bump to 1.0.0 earliest when the rustdoc JSON format has [stabilized](https://rust-lang.zulipchat.com/#narrow/stream/266220-rustdoc/topic/Rustdoc.20JSON.3A.20Stabilization.20criteria).

* **0.x.0**: We bump it when
  * There has been backwards incompatible changes.
  * When `public-api` changes how items are rendered. This is because we want people that use the [`cargo test`](https://github.com/cargo-public-api/cargo-public-api#-as-a-ci-check) approach to not have CI break just because they upgrade **0.0.x** version.
  * When the rustdoc JSON parsing code changes in a backwards incompatible way.
  * `public-api` and `cargo-public-api` does not need to have the same 0.x.0 version, but see the note below on bumping `MINIMUM_NIGHTLY_RUST_VERSION`.

* **0.0.x**: We bump it whenever we want to make a release but we don't have to/want to bump 0.x.0

### Bumping `MINIMUM_NIGHTLY_RUST_VERSION`

If `MINIMUM_NIGHTLY_RUST_VERSION` must be bumped then
* update the [compatibility matrix](https://github.com/cargo-public-api/cargo-public-api#compatibility-matrix)
* by necessity both `cargo-public-api` and `public-api` must be bumped to the same version to make the [compatibility matrix](https://github.com/cargo-public-api/cargo-public-api#compatibility-matrix) consistent.
* bump it in `cargo-public-api` [installation instructions](https://github.com/cargo-public-api/cargo-public-api#installation)
* ```
  rm cargo-public-api/MINIMUM_NIGHTLY_RUST_VERSION_FOR_TESTS
  ```

## How to release

### `cargo-public-api`

1. First release `rustup-toolchain`, `rustdoc-json` and `public-api` if needed, in that order. See below. Note: Because of circular dependencies you must release one helper package at a time from `main`. You can't update all crates in a single commit.
1. Run
   ```
   tag=$(git tag --sort=-creatordate | grep ^v | head -n1) ; git diff $tag
   ```
   to see what is new in the release.
1. For each PR included in the release, adjust its `[category-*]` according to [release.yml](https://github.com/cargo-public-api/cargo-public-api/blob/main/.github/release.yml) and tweak the PR title if necessary to turn it into good release note entry.
1. Preview the [auto-generated release notes](https://github.com/cargo-public-api/cargo-public-api.github.io/blob/main/release-notes-preview.md) which our CI [continuously](https://github.com/cargo-public-api/cargo-public-api/actions/workflows/Preview-release-notes.yml) updates.
    * The target audience for the release notes is users of the `cargo-public-api` CLI. But we also want to credit contributors to our libraries with a mention in the release notes, so we should also include such PRs in the release notes. For libs we also want to update the corresponding `CHANGELOG.md` file though.
1. Bump version with
   ```
   cargo set-version -p cargo-public-api x.y.z
   ```
    * If you bump 0.x.0 version, also update the [compatibility matrix](https://github.com/cargo-public-api/cargo-public-api#compatibility-matrix).
    * If `MINIMUM_NIGHTLY_RUST_VERSION` must be bumped, see [notes](./RELEASE.md#bumping-minimum_nightly_rust_version).
1. Run https://github.com/cargo-public-api/cargo-public-api/actions/workflows/Release-cargo-public-api.yml workflow from `main` ([instructions](https://github.com/cargo-public-api/cargo-public-api/blob/main/docs/development.md#how-to-trigger-main-branch-workflow))
1. Done!

### `public-api`

1. Run
   ```
   tag=$(git tag --sort=-creatordate | grep ^public-api-v | head -n1) ; git diff $tag -- public-api/
   ```
   to see if a release is needed.
1. Update `public-api/CHANGELOG.md`
1. Bump version with
   ```
   cargo set-version -p public-api x.y.z
   ```
    * If you bump 0.x.0 version, also update the [compatibility matrix](https://github.com/cargo-public-api/cargo-public-api#compatibility-matrix).
    * If `MINIMUM_NIGHTLY_RUST_VERSION` must be bumped, see [notes](./RELEASE.md#bumping-minimum_nightly_rust_version).
1. Run https://github.com/cargo-public-api/cargo-public-api/actions/workflows/Release-public-api.yml workflow from `main` ([instructions](https://github.com/cargo-public-api/cargo-public-api/blob/main/docs/development.md#how-to-trigger-main-branch-workflow))
1. Done!

### `rustdoc-json`

1. Run
   ```
   tag=$(git tag --sort=-creatordate | grep ^rustdoc-json-v | head -n1) ; git diff $tag -- rustdoc-json/
   ```
   to see if a release is needed.
1. Update `rustdoc-json/CHANGELOG.md`
1. Bump version with
   ```
   cargo set-version -p rustdoc-json x.y.z
   ```
1. Run https://github.com/cargo-public-api/cargo-public-api/actions/workflows/Release-rustdoc-json.yml workflow from `main` ([instructions](https://github.com/cargo-public-api/cargo-public-api/blob/main/docs/development.md#how-to-trigger-main-branch-workflow))
1. Done!

### `rustup-toolchain`

1. Run
   ```
   tag=$(git tag --sort=-creatordate | grep ^rustup-toolchain-v | head -n1) ; git diff $tag -- rustup-toolchain/
   ```
   to see if a release is needed.
1. Update `rustup-toolchain/CHANGELOG.md`
1. Bump version with
   ```
   cargo set-version -p rustup-toolchain x.y.z
   ```
1. Run https://github.com/cargo-public-api/cargo-public-api/actions/workflows/Release-rustup-toolchain.yml workflow from `main` ([instructions](https://github.com/cargo-public-api/cargo-public-api/blob/main/docs/development.md#how-to-trigger-main-branch-workflow))
1. Done!

## How to trigger main branch workflow

1. Go to https://github.com/cargo-public-api/cargo-public-api/actions and select workflow in the left column
1. Click the **Run workflow ▼** button to the right
1. Make sure the `main` branch is selected
1. Click **Run workflow**
1. Wait for the workflow to complete
