//! Simple wrapper around the library. For a much more sophisticated CLI, see
//! <https://github.com/Enselic/cargo-public-api/blob/main/cargo-public-api/src/main.rs>.

// deny in CI, only warn here
#![warn(clippy::all, clippy::pedantic)]

use std::io::{stdout, ErrorKind, Write};
use std::path::{Path, PathBuf};

use public_api::{PublicApi, MINIMUM_NIGHTLY_RUST_VERSION};

#[derive(thiserror::Error, Debug)]
#[allow(clippy::enum_variant_names)]
enum Error {
    #[error(transparent)]
    PublicApiError(#[from] public_api::Error),
    #[error(transparent)]
    StdIoError(#[from] std::io::Error),
    #[error("{0}")]
    Message(String),
}

type Result<T> = std::result::Result<T, Error>;

/// Use `cargo-public-api` for a much richer set of command line options.
#[derive(Default)]
struct Args {
    help: bool,
    simplified: bool,
    files: Vec<PathBuf>,
}

fn main_() -> Result<()> {
    let args = args();

    let files = &args.files;
    if args.help || files.is_empty() || files.len() > 2 {
        print_usage()?;
    } else if files.len() == 1 {
        let path = &files[0];
        print_public_api(path, &args)?;
    } else {
        Err(Error::Message("Diffing support has been removed from the `public-api` bin (but the library still supports it of course). \
        Please use `cargo-public-api` instead for CLI diffing. It is much better.".into()))?;
    }

    Ok(())
}

fn print_public_api(path: &Path, args: &Args) -> Result<()> {
    for public_item in public_api_from_args(path, args)?.items() {
        writeln!(std::io::stdout(), "{public_item}")?;
    }

    Ok(())
}

fn print_usage() -> std::io::Result<()> {
    writeln!(
        stdout(),
        "public-api v{}

Requires at least {}.

NOTE: See https://github.com/Enselic/cargo-public-api for a convenient cargo
wrapper around this program (or to be precise; library) that does everything
automatically.

If you insist of using this low-level utility and thin wrapper, you run it like this:

    public-api <RUSTDOC_JSON_FILE>

where RUSTDOC_JSON_FILE is the path to the output of

    cargo +nightly rustdoc --lib -- -Z unstable-options --output-format json

which you can find in

    ./target/doc/${{CRATE}}.json

",
        env!("CARGO_PKG_VERSION"),
        MINIMUM_NIGHTLY_RUST_VERSION,
    )
}

/// Helper to parse args.
///
/// Note: I want this Rust package to be simple and without unnecessary
/// dependencies and without the need to select features. For that reason I
/// currently consider it undesirable to for example make this utility depend on
/// `clap` or `anyhow`.
///
/// The convenient wrapper <https://github.com/Enselic/cargo-public-api>
/// depends on both `clap` and `anyhow` though which is perfectly fine.
fn args() -> Args {
    let mut args = Args::default();

    for arg in std::env::args_os().skip(1) {
        if arg == "--simplified" {
            args.simplified = true;
        } else if arg == "--help" || arg == "-h" {
            args.help = true;
        } else {
            args.files.push(PathBuf::from(arg));
        }
    }

    args
}

fn public_api_from_args(path: &Path, args: &Args) -> public_api::Result<PublicApi> {
    public_api::Builder::from_rustdoc_json(path.to_owned())
        .omit_blanket_impls(args.simplified)
        .omit_auto_trait_impls(args.simplified)
        .sorted(true)
        .build()
}

/// Wrapper to handle <https://github.com/rust-lang/rust/issues/46016>
fn main() -> Result<()> {
    match main_() {
        Err(Error::StdIoError(e)) if e.kind() == ErrorKind::BrokenPipe => std::process::exit(141),
        result => result,
    }
}
