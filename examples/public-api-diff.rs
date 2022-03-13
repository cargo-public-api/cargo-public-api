use std::{error::Error, fs::read_to_string, io::stdout};

use public_items::{diff::PublicItemsDiff, Options};

fn main() -> Result<(), Box<dyn Error>> {
    let old = public_items::public_items_from_rustdoc_json_str(
        &read_to_string("./tests/rustdoc_json/public_items-v0.0.4.json")?,
        Options::default(),
    )?;

    let new = public_items::public_items_from_rustdoc_json_str(
        &read_to_string("./tests/rustdoc_json/public_items-v0.0.5.json")?,
        Options::default(),
    )?;

    let diff = PublicItemsDiff::between(old, new);
    diff.print_with_headers(&mut stdout(), "Removed:", "Changed:", "Added:")?;

    Ok(())
}
