use std::error::Error;

use public_api::{diff::PublicApiDiff, Options, PublicApi};

fn main() -> Result<(), Box<dyn Error>> {
    let options = Options::default();

    let old_json = rustdoc_json::Builder::default()
        .toolchain(String::from("nightly"))
        .manifest_path("test-apis/example_api-v0.1.0/Cargo.toml")
        .build()?;
    let old = PublicApi::from_rustdoc_json(old_json, options)?;

    let new_json = rustdoc_json::Builder::default()
        .toolchain(String::from("nightly"))
        .manifest_path("test-apis/example_api-v0.2.0/Cargo.toml")
        .build()?;
    let new = PublicApi::from_rustdoc_json(new_json, options)?;

    let diff = PublicApiDiff::between(old, new);
    println!("{diff:#?}");

    Ok(())
}
