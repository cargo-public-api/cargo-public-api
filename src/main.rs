use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

fn main() -> Result<()> {
    let crate_root = crate_root()?;
    let mut manifest_path = crate_root.clone();
    manifest_path.push("Cargo.toml");

    // Invoke `cargo doc` to build rustdoc JSON
    build_rustdoc_json(&manifest_path)?;

    // Use the `public_items` crate to list all public items (the public API)
    let lib_name = package_name(&manifest_path)?;
    print_public_api_items(&rustdoc_json_path_for_name(&crate_root, &lib_name))?;

    Ok(())
}

/// Returns the root of the crate to analyze. Defaults to current dir. Otherwise
/// uses the path of the first argument to the program.
fn crate_root() -> Result<PathBuf> {
    Ok(match std::env::args_os().nth(1) {
        Some(crate_root) if crate_root != "public-items" => PathBuf::from(crate_root),
        _ => std::env::current_dir().with_context(|| "Failed to get current dir")?,
    })
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

/// Returns `./target/doc/crate_name.json`. Also takes care of transforming
/// `crate-name` to `crate_name`.
fn rustdoc_json_path_for_name(crate_root: &Path, lib_name: &str) -> PathBuf {
    let mut rustdoc_json_path = crate_root.to_owned();
    rustdoc_json_path.push("target");
    rustdoc_json_path.push("doc");
    rustdoc_json_path.push(lib_name.replace("-", "_"));
    rustdoc_json_path.set_extension("json");
    rustdoc_json_path
}

/// Prints all public API items. Sorted.
fn print_public_api_items(path: &Path) -> Result<()> {
    let rustdoc_json = &std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read rustdoc JSON at {:?}", path))?;

    let public_items = public_items::sorted_public_items_from_rustdoc_json_str(rustdoc_json)
        .with_context(|| format!("Failed to parse rustdoc JSON at {:?}", path))?;

    for public_item in public_items {
        writeln!(std::io::stdout(), "{}", public_item)?;
    }

    Ok(())
}
