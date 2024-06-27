use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let json_path = rustdoc_json::Builder::default()
        .toolchain("nightly")
        .manifest_path("testsuite/test-apis/example_api-v0.2.0/Cargo.toml")
        .build()?;

    let public_api = public_api::Builder::from_rustdoc_json(json_path).build()?;

    for public_item in public_api.items() {
        println!("{public_item}");
    }

    Ok(())
}
