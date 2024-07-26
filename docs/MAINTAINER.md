# Maintainer guidelines

Here are some guidelines if you are a maintainer:

**A.** Prefer creating PRs when making a change to ensure CI passes before merge. Prefer waiting on code review for non-trivial changes.

**B.** If a change is low-risk, uncontroversial, and should not end up in the automatically generated changelog for releases, it is fine to push directly to main without going through a PR, CR, and CI pipeline. But please run `scripts/run-ci-locally.sh` locally before pushing. And if CI unexpectedly fails after push, please fix it as soon as possible.

**C.** Never manually `cargo publish`. See 'How to release' below.

**D.** Always keep the main branch in a releasable state. This ensures that we can spontaneously and frequently make releases.

**E.** Avoid having large and long-lived branches. That increases the risk of future merge conflicts and sadness. Prefer many, small, incremental, short-lived PRs that is regularly merged to main.

## Performance

Correct output is much more important than fast code. If we need to regress on performance to e.g. fix a bug, we fix the bug. Performance is secondary. That said, it can be interesting to keep track of performance. See https://github.com/cargo-public-api/cargo-public-api-benchmarks for some tooling regarding this.

## Release strategy

The release philosophy of this project is that it is perfectly fine to make more than one release per week, if circumstances makes that sensible. Why should users have to wait for even a single bugfix? It is better to release whenever there is something new. But sometimes it makes sense to wait 1-2 weeks to make a release, for example to batch up some ongoing PRs. Or sometimes you just feel like doing it that way.

There is one external event that usually means we want to make a release as soon as possible, ideally the same day: When the rustdoc JSON format in nightly changes from one day to the next. If this happens, our Nightly CI job will detect it. If we don't make a new release, users that follows the installation instructions in README.md will see `cargo public-api` failures, because the `cargo public-api` will not know how to parse the rustdoc JSON format of latest nightly.

## How to release

Please see [RELEASE.md](./RELEASE.md).

# Maintainer tips

## Debugging with `tracing`

E.g.

```sh
RUST_LOG=debug cargo run -- --manifest-path test-apis/example_api-v0.1.0/Cargo.toml -sss
```

See the [tracing](https://docs.rs/tracing) docs for more info.

## Finding flaky tests

Run this (WARNING: destructive) command for a while and then scroll back and look at the output. It will find flakiness both in the case of requiring a clean build, and in case of requiring an incremental build. You can also remove the `git clean` of course.
```bash
while true ; do git clean -xdf ; cargo --quiet test ; cargo --quiet test ; sleep 1 ; done | grep -v -e '0 failed' -e 'running [0-9]\+ test'
```

## Minimal dev env

If you get problems in your regular dev env it can help to debug in a minimal environment with minimal variability:

```sh
docker run -it ubuntu
```

```sh
apt-get update -y && apt-get install -y build-essential libssl-dev pkg-config curl git zsh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"
cargo install cargo-audit
cargo install cargo-deny
rustup install nightly --profile minimal
git clone https://github.com/cargo-public-api/cargo-public-api.git
cd cargo-public-api/
./scripts/run-ci-locally.sh
```
