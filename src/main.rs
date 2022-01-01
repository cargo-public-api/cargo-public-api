use std::io::Write;
use std::path::Path;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    match std::env::args_os().nth(1) {
        Some(path) => print_public_api_items(Path::new(&path))?,
        _ => print_usage()?,
    }

    Ok(())
}

fn print_public_api_items(path: &Path) -> Result<()> {
    let rustdoc_json = &std::fs::read_to_string(path)?;

    let mut public_items = Vec::from_iter(public_items::from_rustdoc_json_str(rustdoc_json)?);
    public_items.sort();
    for public_item in public_items {
        writeln!(std::io::stdout(), "{}", public_item)?;
    }

    Ok(())
}

fn print_usage() -> std::io::Result<()> {
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
}
