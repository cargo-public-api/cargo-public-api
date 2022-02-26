use std::io::Write;
use std::path::Path;

use public_items::Options;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    let last_arg = std::env::args_os().last();

    if flag_raised("--help") || flag_raised("-h") || last_arg.is_none() {
        print_usage()?;
    } else {
        let mut options = Options::default();
        options.omit_blanket_implementations = flag_raised("--omit-blanket-implementations");
        print_public_api_items(Path::new(&last_arg.unwrap()), options)?;
    }

    Ok(())
}

fn print_public_api_items(path: &Path, options: Options) -> Result<()> {
    let json = &std::fs::read_to_string(path)?;

    for public_item in public_items::sorted_public_items_from_rustdoc_json_str(json, options)? {
        writeln!(std::io::stdout(), "{}", public_item)?;
    }

    Ok(())
}

fn print_usage() -> std::io::Result<()> {
    writeln!(
        std::io::stdout(),
        r"
NOTE: See https://github.com/Enselic/cargo-public-items for a convenient cargo
wrapper around this program (or to be precise; library) that does everything
automatically.

If you insist of using this low-level utility, you run it like this:

   public_items RUSTDOC_JSON_FILE

where RUSTDOC_JSON_FILE is the path to the output of

  RUSTDOCFLAGS='-Z unstable-options --output-format json' cargo +nightly doc --lib --no-deps

which you can find in

  ./target/doc/${{CRATE}}.json

To omit blanket implementations, pass --omit-blanket-implementations.
"
    )
}

fn flag_raised(flag: &str) -> bool {
    std::env::args_os().into_iter().any(|e| e == flag)
}
