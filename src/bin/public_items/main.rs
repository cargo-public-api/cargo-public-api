use std::ffi::OsStr;
use std::io::Write;
use std::path::Path;

mod cargo_utils;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    match std::env::args_os().nth(1) {
        Some(first_arg) => handle_first_arg(&first_arg)?,
        _ => print_usage()?,
    }

    Ok(())
}

fn handle_first_arg(first_arg: &OsStr) -> Result<()> {
    if first_arg == "--help" || first_arg == "-h" {
        print_usage()?;
    } else if first_arg == "--update" {
        cargo_utils::generate_rustdoc_json_for_current_project()?;
    } else {
        print_public_api_items(Path::new(&first_arg))?;
    }

    Ok(())
}

fn print_public_api_items(path: &Path) -> Result<()> {
    let json = &std::fs::read_to_string(path)?;

    let mut public_items = Vec::from_iter(public_items::public_items_from_rustdoc_json_str(json)?);
    public_items.sort();
    for public_item in public_items {
        writeln!(std::io::stdout(), "{}", public_item)?;
    }

    Ok(())
}

fn print_usage() -> std::io::Result<()> {
    writeln!(
        std::io::stdout(),
        r"
NOTE: See https://github.com/Enselic/cargo-public-items for a convenient cargo
wrapper around this library that does everything automatically.

The particular program you just tried to run is used like this:

   public_items RUSTDOC_JSON_FILE

where RUSTDOC_JSON_FILE is the path to the output of

  RUSTDOCFLAGS='-Z unstable-options --output-format json' cargo +nightly doc --lib --no-deps

which you can find in

  ./target/doc/${{CRATE}}.json
"
    )
}
