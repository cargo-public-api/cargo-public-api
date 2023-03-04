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
  * The `public-api` lib or the `cargo-public-api` CLI has had backwards incompatible changes.
  * When the `public-api` lib changes how items are rendered. This is because we want people that use the [`cargo test`](https://github.com/Enselic/cargo-public-api#-as-a-ci-check) approach to not have CI break just because they upgrade **0.0.x** version.
  * When the rustdoc JSON parsing code changes in a backwards incompatible way.

* **0.0.x**: We bump it whenever we want to make a release but we don't have to/want to bump 0.x.0

## How to release

Please see [RELEASE.md](./RELEASE.md).

# Maintainer tips

## Finding flaky tests

Run this (WARNING: destructive) command for a while and then scroll back and look at the output. It will find flakiness both in the case of requiring a clean build, and in case of requiring an incremental build. You can also remove the `git clean` of course.
```bash
while true ; do git clean -xdf ; cargo --quiet test ; cargo --quiet test ; sleep 1 ; done | grep -v -e '0 failed' -e 'running [0-9]\+ test'
```
