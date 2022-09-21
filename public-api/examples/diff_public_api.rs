use std::{error::Error, fs::read_to_string};

use public_api::{diff::PublicItemsDiff, Options, PublicApi};
use rustdoc_json::BuildOptions;

fn main() -> Result<(), Box<dyn Error>> {
    let options = Options::default();

    let old_json = rustdoc_json::build(
        BuildOptions::default()
            .toolchain(String::from("+nightly"))
            .manifest_path("test-apis/example_api-v0.1.0/Cargo.toml"),
    )?;
    let old = PublicApi::public_api_from_rustdoc_json_str(&read_to_string(old_json)?, options)?;

    let new_json = rustdoc_json::build(
        BuildOptions::default()
            .toolchain(String::from("+nightly"))
            .manifest_path("test-apis/example_api-v0.2.0/Cargo.toml"),
    )?;
    let new = PublicApi::public_api_from_rustdoc_json_str(&read_to_string(new_json)?, options)?;

    let diff = PublicItemsDiff::between(old.items, new.items);
    println!("{:#?}", diff);

    Ok(())
}
