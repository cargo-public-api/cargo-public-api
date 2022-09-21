use std::{error::Error, fs::read_to_string};

use public_api::{Options, PublicApi};

use rustdoc_json::BuildOptions;

fn main() -> Result<(), Box<dyn Error>> {
    let json_path = rustdoc_json::build(
        BuildOptions::default()
            .toolchain(String::from("+nightly"))
            .manifest_path("test-apis/example_api-v0.2.0/Cargo.toml"),
    )?;

    let public_api = PublicApi::public_api_from_rustdoc_json_str(
        &read_to_string(&json_path)?,
        Options::default(),
    )?;

    for public_item in public_api.items {
        println!("{}", public_item);
    }

    Ok(())
}
