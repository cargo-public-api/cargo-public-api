# Maintainer guidelines

Here are some guidelines if you are a maintainer:

**A.** Prefer creating PRs when making a change to ensure CI passes before merge. Prefer waiting on code review for non-trivial changes.

**B.** If a change is low-risk, uncontroversial, and should not end up in the automatically generated changelog for releases, it is fine to push directly to main without going through a PR, CR, and CI pipeline. But please run `scripts/run-ci-locally.sh` locally before pushing. And if CI unexpectedly fails after push, please fix it as soon as possible.

**C.** Never manually `cargo publish`. See 'How to release' below.

**D.** Always keep the main branch in a releasable state. This ensures that we can spontaneously and frequently make releases.

**E.** Avoid having large and long-lived branches. That increases the risk of future merge conflicts and sadness. Prefer many, small, incremental, short-lived PRs that is regularly merged to main.

## Release strategy

The release philosophy of this project is that it is perfectly fine to make more than one release per week, if circumstances makes that sensible. Why should users have to wait for even a single bugfix? It is better to release whenever there is something new. But sometimes it makes sense to wait 1-2 weeks to make a release, for example to batch up some ongoing PRs. Or sometimes you just feel like doing it that way.

There is one external event that usually means we want to make a release as soon as possible, ideally the same day: When the rustdoc JSON format in nightly changes from one day to the next. If this happens, our Nightly CI job will detect it. If we don't make a new release, users that follows the installation instructions in README.md will see `cargo public-api` failures, because the `cargo public-api` will not know how to parse the rustdoc JSON format of latest nightly.

## Versioning strategy

For `public-api` and `cargo-public-api` (which always have the same version number):

* **x.0.0**: We bump to 1.0.0 earliest when the rustdoc JSON format has stabilized, which probably will take many more months, maybe years.

* **0.x.0**: We bump it when
  * The `public_api` lib or the `cargo-public-api` CLI has had backwards incompatible changes.
  * When the rustdoc JSON parsing code changes in a backwards incompatible way.

* **0.0.x**: We bump it whenever we want to make a release but we don't have to/want to bump 0.x.0

## How to release

### `public-api` and `cargo-public-api`

1. First release `rustdoc-json` if needed. See below.
1. Create a PR that targets `main` that
    1. Bumps to the same `version` in
        * **public-api/Cargo.toml** `[package]`
        * **cargo-public-api/Cargo.toml** `[package]`
        * **cargo-public-api/Cargo.toml** `[dependencies.public-api]`
    2. If you bump 0.x.0 version, also update the [compatibility matrix](https://github.com/Enselic/cargo-public-api#compatibility-matrix).
    1. If `MINIMUM_RUSTDOC_JSON_VERSION` must be bumped, bump it. If you bump it, also bump it in
        * [compatibility matrix](https://github.com/Enselic/cargo-public-api#compatibility-matrix)
        * `cargo-public-api` [installation instructions](https://github.com/Enselic/cargo-public-api#installation)
        * `public-api` [installation instructions](https://github.com/Enselic/cargo-public-api/tree/main/public-api#usage)

1. Preview what the auto-generated release-notes will look like by triggering [this](https://github.com/Enselic/cargo-public-api/actions/workflows/Peek-release-notes.yml)
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

1. Bump the `version` in **rustdoc-json/Cargo.toml** and the dependencies declared in **public-api/Cargo.toml** and **cargo-public-api/Cargo.toml**.
1. Run https://github.com/Enselic/cargo-public-api/actions/workflows/Release-rustdoc-json.yml workflow from `main` ([instructions](https://github.com/Enselic/cargo-public-api/blob/main/docs/development.md#how-to-trigger-main-branch-workflow))
1. Double-check that the release ended up at https://crates.io/crates/rustdoc-json/versions
1. Done!

## How to trigger main branch workflow

1. Go to https://github.com/Enselic/cargo-public-api/actions and select workflow in the left column
1. Click the **Run workflow ▼** button to the right
1. Make sure the `main` branch is selected
1. Click **Run workflow**
1. Wait for the workflow to complete
