use std::{error::Error, fs::read_to_string};

use public_api::{Options, PublicApi};

fn main() -> Result<(), Box<dyn Error>> {
    let json_path = rustdoc_json::Builder::default()
        .toolchain(String::from("nightly"))
        .manifest_path("test-apis/example_api-v0.2.0/Cargo.toml")
        .build()?;

    let public_api =
        PublicApi::from_rustdoc_json_str(&read_to_string(&json_path)?, Options::default())?;

    for public_item in public_api.items {
        println!("{}", public_item);
    }

    Ok(())
}
