//! The cargo test framework can't capture stderr from child processes. This
//! little program enables us to test if verbose output works.

fn main() {
    rustdoc_json::Builder::default()
        .manifest_path("tests/test_crates/test_crate/Cargo.toml")
        .verbose(std::env::args().nth(1) == Some("--verbose".to_owned()))
        .build()
        .unwrap();
}
