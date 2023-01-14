# You probably want the CLI

This is a low-level Rust library. You probably want to use the high-level and convenient cargo subcommand: [`cargo public-api`](https://github.com/Enselic/cargo-public-api).

# public-api

List and diff the public API of Rust library crates by analyzing rustdoc JSON output files from the nightly toolchain.

This library is the backbone of [`cargo public-api`](https://github.com/Enselic/cargo-public-api).

## Usage

### In CI

Please see [here](../README.md#-as-a-ci-check).

### CLI

The library comes with a thin bin wrapper that can be used to explore the capabilities of this library.

Build and install the thin bin wrapper with a recent `stable` Rust toolchain:

```console
$ cargo install --locked public-api
```

Ensure a [recent enough](https://github.com/Enselic/cargo-public-api#compatibility-matrix) version of the `nightly` toolchain is installed so you can build up-to-date **rustdoc JSON** files:
```
$ rustup install --profile minimal nightly
```
#### List the public API

First you need to build **rustdoc JSON** for your own Rust library:
```console
$ cd ~/src/your_library
$ cargo +nightly rustdoc --lib -- -Z unstable-options --output-format json
```

Now you can list all items that form the public API of your Rust library:
```console
$ public-api ./target/doc/your_library.json
pub mod your_library
pub struct Foo
...
```

#### Diff the public API

Seriously, use `cargo public-api` for diffing. It has many more diffing capabilities and features. If you insist, you _can_ use `public-api` for diffing. Pass two different **rustdoc JSON** files for two different versions of your library to print the public API diff between them:

```console
$ public-api ./target/doc/your_library.old.json  ./target/doc/your_library.json
Removed:
(nothing)

Changed:
-pub fn your_library::function(arg: bool)
+pub fn your_library::function(arg: usize)

Added:
(nothing)
```

## Library documentation

Documentation can be found at [docs.rs](https://docs.rs/public-api/latest/public_api/index.html) as usual. There are also some simple [examples](https://github.com/Enselic/cargo-public-api/tree/main/public-api/examples) on how to use the library. The code for the [thin bin wrapper](https://github.com/Enselic/cargo-public-api/blob/main/public-api/src/main.rs) might also be of interest. Don't forget to learn how to use this library in CI via [`cargo test`](https://github.com/Enselic/cargo-public-api#-as-a-ci-check)

## Changelog

Please refer to [CHANGELOG.md](https://github.com/Enselic/cargo-public-api/blob/main/public-api/CHANGELOG.md)
