## Tips to work on this tool

This project shares kinship with [`public-api`](https://github.com/Enselic/public-api). Here follows some tips on how to make it easier to work with both projects. This guides assumes you have cloned `public-api` to `~/src/public-api` and [`cargo-public-api`](https://github.com/Enselic/cargo-public-api) to `~/src/cargo-public-api`.

### Make `cargo public-api` use local changes of `public-api`

Uncomment
```toml
# path = "/Users/martin/src/public-api"
```
in `~/src/cargo-public-api/Cargo.toml` and update the path so it fits your system.

### Run local copy of `cargo-public-api` on an arbitrary crate

There are two ways. You can either do:
```
% cd ~/src/arbitrary-crate
% cargo run --manifest-path ~/src/cargo-public-api/Cargo.toml
```
or you can do
```
% cd ~/src/cargo-public-api
% cargo run -- --manifest-path ~/src/arbitrary-crate/Cargo.toml
```
In the first case `--manifest-path` is interpreted by `cargo` itself, and in the second case `--manifest-path` is interpreted by `cargo-public-api`.

NOTE: The second way does not work with `--diff-git-checkouts` yet.

You can also combine both ways:
```
% cd /does/not/matter
% cargo run --manifest-path ~/src/cargo-public-api/Cargo.toml -- --manifest-path ~/src/arbitrary-crate/Cargo.toml
```
