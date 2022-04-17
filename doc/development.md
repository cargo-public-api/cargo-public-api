# Code coverage

Exploring code coverage is a good way to ensure we have broad enough tests. This is the command I use personally to get started:

```bash
cargo llvm-cov --html && open target/llvm-cov/html/index.html
```

Which obviously requires you to have done `cargo install cargo-llvm-cov` first.
