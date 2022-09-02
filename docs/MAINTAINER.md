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

## How to release

### `public-api` and `cargo-public-api`

1. First release `rustdoc-json` if needed. See below.
1. Bump to the same `version` in **public-api/Cargo.toml** and **cargo-public-api/Cargo.toml** (including the dependency on `public-api`), and push to `main`. If you bump 0.x.0 version, also update the [compatibility matrix](https://github.com/Enselic/cargo-public-api#compatibility-matrix).
1. If `MINIMUM_RUSTDOC_JSON_VERSION` must be bumped, bump it. If you bump it, also bump it in [installation instruction](https://github.com/Enselic/cargo-public-api#installation) and the [compatibility matrix](https://github.com/Enselic/cargo-public-api#compatibility-matrix).
1. Label PRs that should not be mentioned in the release notes with `[exclude-from-release-notes]`. Label PRs that should be in the "New Features" section in the auto-generated release notes with `[enhancement]`. You can preview release notes by triggering [this](https://github.com/Enselic/cargo-public-api/actions/workflows/Peek-release-notes.yml).
1. Run https://github.com/Enselic/cargo-public-api/actions/workflows/Release.yml workflow from `main` ([instructions](https://github.com/Enselic/cargo-public-api/blob/main/docs/development.md#how-to-trigger-main-branch-workflow))
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
