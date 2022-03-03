use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use public_items::{Options, PublicItem};

use clap::Parser;

const MIN_NIGHTLY: &str = "nightly-2022-02-23";

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// Raise this flag to make items part of blanket implementations such as
    /// `impl<T> Any for T`, `impl<T> Borrow<T> for T`, and `impl<T, U> Into<U>
    /// for T where U: From<T>` be included in the list of public items of a
    /// crate.
    ///
    /// Blanket implementations are not included by default since the the vast
    /// majority of users will find the presence of these items to just
    /// constitute noise, even if they formally are part of the public API of a
    /// crate.
    #[clap(long)]
    with_blanket_implementations: bool,

    /// Path to `Cargo.toml`.
    #[clap(long, default_value = "Cargo.toml")]
    manifest_path: PathBuf,
}

fn main() -> Result<()> {
    let args = get_args();

    // Invoke `cargo doc` to build rustdoc JSON
    build_rustdoc_json(&args.manifest_path)?;

    // Use the `public_items` crate to list all public items (the public API)
    // after settings things up a bit
    let target_directory = get_target_directory(&args.manifest_path)?;
    let lib_name = package_name(&args.manifest_path)?;
    let json_path = rustdoc_json_path_for_name(&target_directory, &lib_name);
    let options = get_options(&args);
    let public_items = collect_public_api_items(&json_path, options)?;
    print_public_items(&public_items)?;

    Ok(())
}

/// Get CLI args via `clap` while also handling when we are invoked as a cargo
/// subcommand
fn get_args() -> Args {
    // If we are invoked by cargo as `cargo public-items`, the second arg will
    // be "public-items". Remove it before passing args on to clap. If we are
    // not invoked as a cargo subcommand, it will not be part of args at all, so
    // it is safe to filter it out also in that case.
    let args = std::env::args_os().filter(|x| x != "public-items");

    Args::parse_from(args)
}

/// Synchronously generate the rustdoc JSON for the library crate in the current
/// directory.
fn build_rustdoc_json(crate_root: &Path) -> Result<()> {
    let mut command = std::process::Command::new("cargo");
    command.args(["+nightly", "doc", "--lib", "--no-deps"]);
    command.arg("--manifest-path");
    command.arg(crate_root);
    command.env("RUSTDOCFLAGS", "-Z unstable-options --output-format json");
    command.spawn()?.wait()?;

    Ok(())
}

/// Figures out the name of the library crate in the current directory by
/// looking inside `Cargo.toml`
fn package_name(path: impl AsRef<Path>) -> Result<String> {
    let manifest = cargo_toml::Manifest::from_path(&path)
        .with_context(|| format!("Failed to parse manifest at {:?}", path.as_ref()))?;
    Ok(manifest
        .package
        .expect("[package] is declared in Cargo.toml")
        .name)
}

/// Typically returns the absolute path to the regular cargo `./target` directory.
fn get_target_directory(manifest_path: &Path) -> Result<PathBuf> {
    let mut metadata_cmd = cargo_metadata::MetadataCommand::new();
    metadata_cmd.manifest_path(&manifest_path);
    let metadata = metadata_cmd.exec()?;

    Ok(metadata.target_directory.as_std_path().to_owned())
}

/// Figure out what [`Options`] to pass to
/// [`public_items::sorted_public_items_from_rustdoc_json_str`] based on our
/// [`Args`]
fn get_options(args: &Args) -> Options {
    let mut options = Options::default();
    options.with_blanket_implementations = args.with_blanket_implementations;
    options
}

/// Returns `./target/doc/crate_name.json`. Also takes care of transforming
/// `crate-name` to `crate_name`.
fn rustdoc_json_path_for_name(target_directory: &Path, lib_name: &str) -> PathBuf {
    let mut rustdoc_json_path = target_directory.to_owned();
    rustdoc_json_path.push("doc");
    rustdoc_json_path.push(lib_name.replace('-', "_"));
    rustdoc_json_path.set_extension("json");
    rustdoc_json_path
}

/// Collects public items from a given rustdoc JSON path.
fn collect_public_api_items(path: &Path, options: Options) -> Result<Vec<PublicItem>> {
    let rustdoc_json = &std::fs::read_to_string(path).with_context(|| {
        format!(
            "Failed to read rustdoc JSON at {:?}.\n\
             This version of `cargo public-items` requires at least:\n\n    {}.\n\n\
             If you have that, it might be `cargo public-items` that is out of date. Try\n\
             to install the latest versions with `cargo install cargo-public-items`,",
            path, MIN_NIGHTLY
        )
    })?;

    public_items::sorted_public_items_from_rustdoc_json_str(rustdoc_json, options)
        .with_context(|| format!("Failed to parse rustdoc JSON at {:?}", path))
}

/// Prints all public items.
fn print_public_items(public_items: &[PublicItem]) -> Result<()> {
    for public_item in public_items {
        writeln!(std::io::stdout(), "{}", public_item)?;
    }

    Ok(())
}
