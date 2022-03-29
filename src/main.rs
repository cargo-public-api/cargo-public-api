use std::io::{stdout, Write};
use std::path::{Path, PathBuf};

use public_items::diff::PublicItemsDiff;
use public_items::{public_items_from_rustdoc_json_str, Options, MINIMUM_RUSTDOC_JSON_VERSION};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    let mut options = Options::default();
    options.with_blanket_implementations = flag_raised("--with-blanket-implementations");
    options.sorted = true;

    let mut args = std::env::args_os();
    if flag_raised("--help") || flag_raised("-h") || args.len() <= 1 || args.len() > 3 {
        print_usage()?;
    } else if args.len() == 2 {
        args.next();
        let path = PathBuf::from(args.next().unwrap());
        print_public_items(&path, options)?;
    } else if args.len() == 3 {
        args.next();
        let old = PathBuf::from(args.next().unwrap());
        let new = PathBuf::from(args.next().unwrap());
        print_public_items_diff(&old, &new, options)?;
    }

    Ok(())
}

fn print_public_items(path: &Path, options: Options) -> Result<()> {
    let json = &std::fs::read_to_string(path)?;

    for public_item in public_items_from_rustdoc_json_str(json, options)? {
        writeln!(std::io::stdout(), "{}", public_item)?;
    }

    Ok(())
}

fn print_public_items_diff(old: &Path, new: &Path, options: Options) -> Result<()> {
    let old_json = std::fs::read_to_string(old)?;
    let old_items = public_items_from_rustdoc_json_str(&old_json, options)?;

    let new_json = std::fs::read_to_string(new)?;
    let new_items = public_items_from_rustdoc_json_str(&new_json, options)?;

    let diff = PublicItemsDiff::between(old_items, new_items);
    print_diff_with_headers(&diff, &mut stdout(), "Removed:", "Changed:", "Added:")?;

    Ok(())
}

fn print_diff_with_headers(
    diff: &PublicItemsDiff,
    w: &mut impl std::io::Write,
    header_removed: &str,
    header_changed: &str,
    header_added: &str,
) -> std::io::Result<()> {
    print_items_with_header(w, header_removed, &diff.removed, |w, item| {
        writeln!(w, "-{}", item)
    })?;
    print_items_with_header(w, header_changed, &diff.changed, |w, item| {
        writeln!(w, "-{}", item.old)?;
        writeln!(w, "+{}", item.new)
    })?;
    print_items_with_header(w, header_added, &diff.added, |w, item| {
        writeln!(w, "+{}", item)
    })?;

    Ok(())
}

fn print_items_with_header<W: std::io::Write, T>(
    w: &mut W,
    header: &str,
    items: &[T],
    print_fn: impl Fn(&mut W, &T) -> std::io::Result<()>,
) -> std::io::Result<()> {
    writeln!(w, "{}", header)?;
    if items.is_empty() {
        writeln!(w, "(nothing)")?;
    } else {
        for item in items {
            print_fn(w, item)?;
        }
    }
    writeln!(w)
}

fn print_usage() -> std::io::Result<()> {
    writeln!(
        stdout(),
        "public_items v{}

Requires at least {}.

NOTE: See https://github.com/Enselic/cargo-public-items for a convenient cargo
wrapper around this program (or to be precise; library) that does everything
automatically.

If you insist of using this low-level utility and thin wrapper, you run it like this:

    public_items <RUSTDOC_JSON_FILE>

where RUSTDOC_JSON_FILE is the path to the output of

    RUSTDOCFLAGS='-Z unstable-options --output-format json' cargo +nightly doc --lib --no-deps

which you can find in

    ./target/doc/${{CRATE}}.json

To diff the public API between two commits, you generate one rustdoc JSON file for each
commit and then pass the path of both files to this utility:

    public_items <RUSTDOC_JSON_FILE_OLD> <RUSTDOC_JSON_FILE_NEW>

To include blanket implementations, pass --with-blanket-implementations.
",
        env!("CARGO_PKG_VERSION"),
        MINIMUM_RUSTDOC_JSON_VERSION,
    )
}

/// Helper to check if a flag is raised in command line args.
///
/// Note: I want this Rust package to be simple and without unnecessary
/// dependencies and without the need to select features. For that reason I
/// currently consider it undesirable to for example make this utility depend on
/// `clap` or `anyhow`.
///
/// The convenient wrapper <https://github.com/Enselic/cargo-public-items>
/// depends on both `clap` and `anyhow` though which is perfectly fine.
fn flag_raised(flag: &str) -> bool {
    std::env::args_os().into_iter().any(|e| e == flag)
}
