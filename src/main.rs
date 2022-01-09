use std::io::Write;
use std::path::{Path, PathBuf};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    // Invoke `cargo doc` to build rustdoc JSON
    build_rustdoc_json()?;

    // Use the `public_items` crate to list all public items (the public API)
    let lib_name = package_name("Cargo.toml")?;
    print_public_api_items(&rustdoc_json_path_for_name(lib_name))?;

    Ok(())
}

/// Synchronously generate the rustdoc JSON for the library crate in the current
/// directory.
fn build_rustdoc_json() -> Result<()> {
    let mut command = std::process::Command::new("cargo");
    command.args(["+nightly", "doc", "--lib", "--no-deps"]);
    command.env("RUSTDOCFLAGS", "-Z unstable-options --output-format json");
    command.spawn()?.wait()?;

    Ok(())
}

/// Figures out the name of the library crate in the current directory by
/// looking inside `Cargo.toml`
fn package_name(path: impl AsRef<Path>) -> Result<String> {
    let manifest = cargo_toml::Manifest::from_path(path)?;
    Ok(manifest
        .package
        .expect("[package] is declared in Cargo.toml")
        .name)
}

/// Returns `./target/doc/crate_name.json`.
fn rustdoc_json_path_for_name(lib_name: String) -> PathBuf {
    let mut rustdoc_json_path = PathBuf::from("./target/doc/");
    rustdoc_json_path.push(lib_name);
    rustdoc_json_path.set_extension("json");
    rustdoc_json_path
}

/// Prints all public API items. Sorted.
fn print_public_api_items(path: &Path) -> Result<()> {
    let rustdoc_json = &std::fs::read_to_string(path)?;

    let mut public_items = Vec::from_iter(public_items::from_rustdoc_json_str(rustdoc_json)?);
    public_items.sort();
    for public_item in public_items {
        writeln!(std::io::stdout(), "{}", public_item)?;
    }

    Ok(())
}
