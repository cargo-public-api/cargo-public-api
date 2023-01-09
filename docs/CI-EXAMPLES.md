# CI Examples

This document describes different ways to make use of `cargo public-api` in CI. For regular usage, see [Usage](../README.md#usage) instead.

## Prevent Accidental Public API Changes

### With a Public API Set in Stone

If the API is set in stone, you can use the `--deny=all` flag together with `diff ...` to deny all kinds of changes (additions, changes, removals) to your public API. A GitHub Actions job to do this for PRs would look something like this:

```yaml
jobs:
  deny-public-api-changes:
    runs-on: ubuntu-latest
    steps:
      # Full git history needed
      - uses: actions/checkout@v3
        with:
          fetch-depth: 0

      # Install nightly (stable is already installed)
      - run: rustup install --profile minimal nightly

      # Install and run cargo public-api and deny any API diff
      - run: cargo install cargo-public-api
      - run: cargo public-api diff ${GITHUB_BASE_REF}..${GITHUB_HEAD_REF} --deny=all
```

See `cargo public-api --help` for more variants of `--deny`.

### With a Changeable Public API

Sometimes you want CI to prevent accidental changes to your public API while still allowing you to easily bless changes to the public API. To do this, first write the current public API to a file:

```bash
cargo +nightly-2022-09-28 public-api > public-api.txt
```

> NOTE: This example uses a fixed nightly toolchain. See [Locking](#locking) for more info.

Then create a CI job that ensures the API remains unchanged, with instructions on how to bless changes. A GitHub Actions job to do so would look something like this:

```yaml
jobs:
  deny-public-api-changes:
    runs-on: ubuntu-latest
    steps:
      # Install nightly (stable is already installed)
      - run: rustup install --profile minimal nightly

      # Install and run cargo public-api and deny any API diff
      - run: cargo install cargo-public-api@0.22.0
      - run: |
          diff -u public-api.txt <(cargo +nightly-2022-08-15 public-api) ||
              (echo '\nFAIL: Public API changed! To bless, `git commit` the result of `cargo +nightly-2022-08-15 public-api > public-api.txt`' && exit 1)
```

#### Locking

Since the rustdoc JSON format is unstable and frequently changes across nightly toolchain versions, and since improvements to `cargo public-api` are regularly released, you probably want to lock against a specific version of `cargo public-api` and a specific version of the nightly toolchain. To find matching versions, consult the [Compatibility Matrix](../README.md#compatibility-matrix). Then use the syntax above to provision CI with these versions.
