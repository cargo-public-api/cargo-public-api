use std::{collections::HashSet, path::Path};

use std::io::prelude::*;

use public_items::Result;

fn main() -> Result<()> {
    match std::env::args_os().nth(1) {
        Some(path) => print_public_api_items(Path::new(&path)),
        _ => print_usage(),
    }
}

fn print_public_api_items(path: &Path) -> Result<()> {
    let public_items = from_rustdoc_json_path(path)?;

    for public_item in public_items {
        println!("{}", public_item);
    }

    Ok(())
}

fn from_rustdoc_json_path(path: &Path) -> Result<HashSet<String>> {
    public_items::from_rustdoc_json_str(&std::fs::read_to_string(path)?)
}

fn print_usage() -> Result<()> {
    writeln!(
        std::io::stdout(),
        "Usage:

   public_items RUSTDOC_JSON_FILE

where RUSTDOC_JSON_FILE is the path to the output of

  RUSTDOCFLAGS='-Z unstable-options --output-format json' cargo +nightly doc --no-deps

which you can find in

  ./target/doc/${{{{CRATE}}}}.json
"
    )
    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("{}", e)).into())
}
