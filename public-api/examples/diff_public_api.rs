use std::error::Error;

use public_api::diff::PublicApiDiff;

fn main() -> Result<(), Box<dyn Error>> {
    let old_json = rustdoc_json::Builder::default()
        .toolchain("nightly")
        .manifest_path("testsuite/test-apis/example_api-v0.1.0/Cargo.toml")
        .build()?;
    let old = public_api::Builder::from_rustdoc_json(old_json).build()?;

    let new_json = rustdoc_json::Builder::default()
        .toolchain("nightly")
        .manifest_path("testsuite/test-apis/example_api-v0.2.0/Cargo.toml")
        .build()?;
    let new = public_api::Builder::from_rustdoc_json(new_json).build()?;

    let diff = PublicApiDiff::between(old, new);
    println!("{diff:#?}");

    Ok(())
}
