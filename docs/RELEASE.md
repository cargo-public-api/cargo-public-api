## Versioning strategy

For `public-api` and `cargo-public-api` we bump

* **x.0.0**
   * no earlier than rustdoc JSON format [stabilization](https://rust-lang.zulipchat.com/#narrow/stream/266220-rustdoc/topic/Rustdoc.20JSON.3A.20Stabilization.20criteria).
* **0.x.0**
   * independently between `public-api` and `cargo-public-api`. [^1]
   * when `public_api::MINIMUM_NIGHTLY_RUST_VERSION` is bumped. [^2]
   * when `public-api` changes how items are rendered. [^3]
   * for other regular semver breaking changes. [^4]
* **0.0.x**
   * whenever we want to make a release but don't have to/want to bump **0.x.0**.

[^1]: But if `public_api::MINIMUM_NIGHTLY_RUST_VERSION` has been bumped then `public-api` and `cargo-public-api` by necessity must bump to the same **0.x.0** version for the [compatibility matrix](https://github.com/cargo-public-api/cargo-public-api#compatibility-matrix).
[^2]: Since we need to be able to add a new row to the [compatibility matrix](https://github.com/cargo-public-api/cargo-public-api#compatibility-matrix).
[^3]: Because otherwise [`CI checks`](https://github.com/cargo-public-api/cargo-public-api#-as-a-ci-check) would fail from **0.0.x** updates.
[^4]: E.g. changes to the `public-api` public API or the `cargo-public-api` CLI.

### When not to release

* If a package had no **x.0.0** or **0.x.0** updates to regular `dependencies` (we disregard `dev-dependencies`) we don't need a new release of that package. [^5]

[^5]: Because there are no technical reasons to do it. On the contrary, it creates unnecessary churn for downstream users.

## How to release

### `cargo-public-api`

1. First release `rustup-toolchain`, `rustdoc-json` and `public-api` if needed, in that order. See below.
1. Create a local branch.
1. Run
   ```
   tag=$(git tag --sort=-creatordate | grep ^v | head -n1) ; git diff $tag
   ```
   to see what is new in the release.
1. Update `CHANGELOG.md`
1. Bump version with
   ```
   cargo set-version -p cargo-public-api x.y.z
   ```
   If **0.x.0** or `public_api::MINIMUM_NIGHTLY_RUST_VERSION` was bumped:
      * Add a new version info entry at [version_info.rs](https://github.com/cargo-public-api/cargo-public-api/blob/main/scripts/release-helper/lib/version_info.rs)
      * Run
        ```
        cargo run --bin update-version-info
        ````
1. Push branch

#### MAINTAINER:

1. Once PR merges, run https://github.com/cargo-public-api/cargo-public-api/actions/workflows/Release-cargo-public-api.yml workflow from `main` ([instructions](#how-to-trigger-main-branch-workflow))
1. Done!

### `public-api`

1. Run
   ```
   tag=$(git tag --sort=-creatordate | grep ^public-api-v | head -n1) ; git diff $tag -- public-api/
   ```
   to see if a release is needed.
1. If changes detected, create a local branch.
1. Update `public-api/CHANGELOG.md`
1. Bump version with
   ```
   cargo set-version -p public-api x.y.z
   ```
   If **0.x.0** or `public_api::MINIMUM_NIGHTLY_RUST_VERSION` was bumped:
      * Update [version_info.rs](https://github.com/cargo-public-api/cargo-public-api/blob/main/scripts/release-helper/lib/version_info.rs)
      * Run
        ```
        cargo run --bin update-version-info
        ````
1. Push branch

#### MAINTAINER:

1. Once PR merges, run https://github.com/cargo-public-api/cargo-public-api/actions/workflows/Release-public-api.yml workflow from `main` ([instructions](#how-to-trigger-main-branch-workflow))
1. Done!

### `rustdoc-json`

1. Run
   ```
   tag=$(git tag --sort=-creatordate | grep ^rustdoc-json-v | head -n1) ; git diff $tag -- rustdoc-json/
   ```
   to see if a release is needed.
1. If changes detected, create a local branch.
1. Update `rustdoc-json/CHANGELOG.md`
1. Bump version with
   ```
   ( cd rustdoc-json ; cargo set-version -p rustdoc-json x.y.z )
   ```
1. Push branch

#### MAINTAINER:

1. Once PR merges, run https://github.com/cargo-public-api/cargo-public-api/actions/workflows/Release-rustdoc-json.yml workflow from `main` ([instructions](#how-to-trigger-main-branch-workflow))
1. Done!

### `rustup-toolchain`

1. Run
   ```
   tag=$(git tag --sort=-creatordate | grep ^rustup-toolchain-v | head -n1) ; git diff $tag -- rustup-toolchain/
   ```
   to see if a release is needed.
1. If changes detected, create a local branch.
1. Update `rustup-toolchain/CHANGELOG.md`
1. Bump version with
   ```
   ( version=0.x.y ; cargo set-version -p rustup-toolchain $version && cd rustup-toolchain && cargo set-version -p rustup-toolchain $version )
   ```
1. Push branch

#### MAINTAINER:

1. Once PR merges, run https://github.com/cargo-public-api/cargo-public-api/actions/workflows/Release-rustup-toolchain.yml workflow from `main` ([instructions](#how-to-trigger-main-branch-workflow))
1. Done!

## How to trigger main branch workflow

1. Go to https://github.com/cargo-public-api/cargo-public-api/actions and select workflow in the left column
1. Click the **Run workflow ▼** button to the right
1. Make sure the `main` branch is selected
1. Click **Run workflow**
1. Wait for the workflow to complete
