# Minimum required Rust version

This project is guaranteed to build the the latest stable Rust toolchain. More specifically, the toolchain that is installed by default on GitHub's `ubuntu-latest` runner. You can see [here](https://github.com/actions/virtual-environments/blob/main/images/linux/Ubuntu2004-Readme.md#rust-tools) what version that currently is.

Note that the toolchain required to build this library is distinct from the toolchain required to generate the rustdoc JSON that this library processes. Rustdoc JSON can currently only be generated with the nightly toolchain.

# Code coverage

Exploring code coverage is a good way to ensure we have broad enough tests. This is the command I use personally to get started:

```bash
cargo llvm-cov --html && open target/llvm-cov/html/index.html
```

Which obviously requires you to have done `cargo install cargo-llvm-cov` first.


# Maintainer guidelines

Here are some guidelines if you are a maintainer:

**A.** Prefer creating PRs when making a change to ensure CI passes before merge. Prefer waiting on code review for non-trivial changes.

**B.** If a change is low-risk and uncontroversial, it is fine to push directly to main without going through a PR and a CI pipeline. But please run `scripts/run-ci-locally.sh` locally before pushing. And if CI unexpectedly fails after push, please fix it as soon as possible.

**C.** Never manually `cargo publish`. See 'How to release' below.

**D.** Always keep the main branch in a releasable state. This ensures that we can spontaneously and frequently make releases.

**E.** Avoid having large and long-lived branches. That increases the risk of future merge conflicts and sadness. Prefer many, small, incremental, short-lived PRs that is regularly merged to main.

## How to release

0. Bump `version` in **Cargo.toml** and push to `main`
1. Go to https://github.com/Enselic/public-api/actions/workflows/Release.yml
2. Click the **Run workflow ▼** button to the right
3. Make sure the `main` branch is selected
4. Click **Run workflow**
5. Wait for the workflow to complete
6. Double-check that the release ended up at https://crates.io/crates/public-api/versions
7. Double-check that the auto-generated release notes for the release at https://github.com/Enselic/public-api/releases is not horribly inaccurate
8. Done!
