//! The cargo test framework can't capture stderr from child processes. This
//! little program enables us to test if stderr can be suppressed with
//! `silent(true)`.

fn main() {
    rustdoc_json::Builder::default()
        .manifest_path("invalid/because/we/want/it/to/fail/Cargo.toml")
        .silent(std::env::args().nth(1) == Some("--silent".to_owned()))
        .build()
        .unwrap();
}
