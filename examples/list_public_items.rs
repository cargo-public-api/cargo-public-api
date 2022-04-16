use std::{error::Error, fs::read_to_string};

use public_items::{public_items_from_rustdoc_json_str, Options};

fn main() -> Result<(), Box<dyn Error>> {
    let public_items = public_items_from_rustdoc_json_str(
        &read_to_string("./tests/rustdoc-json/example_api-v0.2.0.json")?,
        Options::default(),
    )?;

    for public_item in public_items {
        println!("{}", public_item);
    }

    Ok(())
}
