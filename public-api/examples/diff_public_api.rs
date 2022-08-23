use std::{error::Error, fs::read_to_string};

use public_api::{diff::PublicItemsDiff, public_api_from_rustdoc_json_str, Options};

fn main() -> Result<(), Box<dyn Error>> {
    let options = Options::default();

    let old = public_api_from_rustdoc_json_str(
        &read_to_string("./tests/rustdoc-json/example_api-v0.1.0.json")?,
        options,
    )?;

    let new = public_api_from_rustdoc_json_str(
        &read_to_string("./tests/rustdoc-json/example_api-v0.2.0.json")?,
        options,
    )?;

    let diff = PublicItemsDiff::between(old.items, new.items);
    println!("{:#?}", diff);

    Ok(())
}
