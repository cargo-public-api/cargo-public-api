# cargo-public-api

List and diff the public API of Rust library crates between releases and commits. Detect breaking API changes and semver violations. Relies on and automatically builds [rustdoc JSON](https://github.com/rust-lang/rust/issues/76578), for which a recent version of the Rust nightly toolchain must be installed.

# Installation

```bash
# Install cargo-public-api with a recent regular stable Rust toolchain
cargo install cargo-public-api

# Ensure nightly-2022-08-15 or later is installed so cargo-public-api can build rustdoc JSON for you
rustup install nightly
```

# Usage

## List the Public API

This example lists the public API of the ubiquitous `regex` crate. First we clone the repo:

```bash
git clone https://github.com/rust-lang/regex
cd regex
```

Now we can list the public API of `regex` by running

```bash
cargo public-api
```

which will print the public API of `regex` with one line per public item in the API:

<img src="docs/img/list.jpg" alt="colored output of listing a public api">

## Diff the Public API

To diff the API between say **0.2.2** and **0.2.3** of `regex`, use `--diff-git-checkouts 0.2.2 0.2.3` while standing in the git repo. Like this:

```bash
cargo public-api --diff-git-checkouts 0.2.2 0.2.3
```

and the API diff will be printed:

<img src="docs/img/diff.jpg" alt="colored output of diffing a public api">

### Of Your Current Branch

When you make changes to your library you often want to make sure that you do not accidentally change the public API of your library, or that the API change you are making looks like you expect. For this use case, first git commit your work in progress, and then run

```bash
cargo public-api --diff-git-checkouts origin/main your-current-branch
```

which will print the the diff of your public API changes compared to `origin/main`.

### As a CI Check With a Changeable Public API

Sometimes you want CI to prevent accidental changes to your public API while still allowing you to easily bless changes to the public API. To do this, first write the current public API to a file:

```bash
cargo public-api > public-api.txt
```

and then create a CI job that ensures the API remains unchanged, with instructions on how to bless changes. For example, a GitHub Actions job to do so would look something like this:

```yaml
jobs:
  deny-public-api-changes:
    runs-on: ubuntu-latest
    steps:
      # Install nightly (stable is already installed)
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: nightly
          profile: minimal

      # Install and run cargo public-api and deny any API diff
      - run: cargo install cargo-public-api
      - run: |
          diff -u public-api.txt <(cargo public-api) ||
              (echo '\nFAIL: Public API changed! To bless, `git commit` the result of `cargo public-api > public-api.txt`' && exit 1)
```

### As a CI Check With a Public API Set in Stone

If the API is set in stone, another alternative is to use the `--deny=all` flag together with `--diff-git-checkouts`. A GitHub Actions job to do this for PRs would look something like this:

```yaml
jobs:
  deny-public-api-changes:
    runs-on: ubuntu-latest
    steps:
      # Full git history needed
      - uses: actions/checkout@v2
        with:
          fetch-depth: 0

      # Install nightly (stable is already installed)
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: nightly
          profile: minimal

      # Install and run cargo public-api and deny any API diff
      - run: cargo install cargo-public-api
      - run: cargo public-api --diff-git-checkouts ${GITHUB_BASE_REF} ${GITHUB_HEAD_REF} --deny=all
```

See `cargo public-api --help` for more variants of `--deny`.

## Expected Output

Output aims to be character-by-character identical to the textual parts of the regular `cargo doc` HTML output. For example, [this item](https://docs.rs/bat/0.20.0/bat/struct.PrettyPrinter.html#method.input_files) has the following textual representation in the rendered HTML:

```
pub fn input_files<I, P>(&mut self, paths: I) -> &mut Self
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
```

and `cargo public-api` represents this item in the following manner:

```
pub fn bat::PrettyPrinter::input_files<I, P>(&mut self, paths: I) -> &mut Self where I: IntoIterator<Item = P>, P: AsRef<Path>
```

If we normalize by removing newline characters and adding some whitespace padding to get the alignment right for side-by-side comparison, we can see that they are exactly the same, except an irrelevant trailing comma:

```
pub fn                     input_files<I, P>(&mut self, paths: I) -> &mut Self where I: IntoIterator<Item = P>, P: AsRef<Path>,
pub fn bat::PrettyPrinter::input_files<I, P>(&mut self, paths: I) -> &mut Self where I: IntoIterator<Item = P>, P: AsRef<Path>
```

## Blanket Implementations

By default, blanket implementations such as `impl<T> Any for T`, `impl<T> Borrow<T> for T`, and `impl<T, U> Into<U> for T where U: From<T>` are omitted from the list of public items of a crate. For the vast majority of use cases, blanket implementations are not of interest, and just creates noise.

Use `--with-blanket-implementations` if you want to include items of blanket implementations in the output:
```bash
cargo public-api --with-blanket-implementations
```

# Compatibility Matrix

| cargo-public-api | Understands the rustdoc JSON output of  |
| ---------------- | --------------------------------------- |
| v0.14.x          | nightly-2022-08-15 —                    |
| v0.13.x          | nightly-2022-08-10 — nightly-2022-08-14 |
| v0.12.x          | nightly-2022-05-19 — nightly-2022-08-09 |
| v0.10.x          | nightly-2022-03-14 — nightly-2022-05-18 |
| v0.5.x           | nightly-2022-02-23 — nightly-2022-03-13 |
| v0.2.x           | nightly-2022-01-19 — nightly-2022-02-22 |
| v0.0.5           | nightly-2021-10-11 — nightly-2022-01-18 |

# Contributing

See [CONTRIBUTING.md](./docs/CONTRIBUTING.md).

## Maintainers

- [Enselic](https://github.com/Enselic)
- [douweschulte](https://github.com/douweschulte)
