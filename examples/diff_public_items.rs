use std::{error::Error, fs::read_to_string};

use public_items::{diff::PublicItemsDiff, public_items_from_rustdoc_json_str, Options};

fn main() -> Result<(), Box<dyn Error>> {
    let options = Options::default();

    let old = public_items_from_rustdoc_json_str(
        &read_to_string("./tests/rustdoc-json/example_api-v0.1.0.json")?,
        options,
    )?;

    let new = public_items_from_rustdoc_json_str(
        &read_to_string("./tests/rustdoc-json/example_api-v0.2.0.json")?,
        options,
    )?;

    let diff = PublicItemsDiff::between(old, new);
    println!("{:#?}", diff);

    Ok(())
}
