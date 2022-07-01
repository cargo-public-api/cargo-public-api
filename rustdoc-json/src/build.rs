use super::BuildError;

use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    process::Command,
};

/// Run `cargo rustdoc` to produce rustdoc JSON and return the path to the built
/// file.
pub(crate) fn run_cargo_rustdoc(
    toolchain: impl AsRef<OsStr>,
    manifest_path: impl AsRef<Path>,
    quiet: bool,
) -> Result<PathBuf, BuildError> {
    let output = cargo_rustdoc_command(toolchain, &manifest_path, quiet).output()?;
    if output.status.success() {
        rustdoc_json_path_for_manifest_path(manifest_path)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        if stderr.contains("is a virtual manifest, but this command requires running against an actual package in this workspace") {
            Err(BuildError::VirtualManifest(
                manifest_path.as_ref().to_owned(),
            ))
        } else {
            Err(BuildError::General(stderr))
        }
    }
}

/// Construct the `cargo rustdoc` command to use for building rustdoc JSON. The
/// command typically ends up looks something like this:
/// ```bash
/// cargo +nightly rustdoc --lib --manifest-path Cargo.toml -- -Z unstable-options --output-format json --cap-lints warn
/// ```
fn cargo_rustdoc_command(
    toolchain: impl AsRef<OsStr>,
    manifest_path: impl AsRef<Path>,
    quiet: bool,
) -> Command {
    let mut command = Command::new("cargo");
    command.arg(toolchain.as_ref());
    command.arg("rustdoc");
    command.arg("--lib");
    if quiet {
        command.arg("--quiet");
    }
    command.arg("--manifest-path");
    command.arg(manifest_path.as_ref());
    command.arg("--");
    command.args(["-Z", "unstable-options"]);
    command.args(["--output-format", "json"]);
    command.args(["--cap-lints", "warn"]);
    command
}

/// Returns `./target/doc/crate_name.json`. Also takes care of transforming
/// `crate-name` to `crate_name`.
fn rustdoc_json_path_for_manifest_path(
    manifest_path: impl AsRef<Path>,
) -> Result<PathBuf, BuildError> {
    let target_dir = target_directory(&manifest_path)?;
    let lib_name = package_name(&manifest_path)?;

    let mut rustdoc_json_path = target_dir;
    rustdoc_json_path.push("doc");
    rustdoc_json_path.push(lib_name.replace('-', "_"));
    rustdoc_json_path.set_extension("json");
    Ok(rustdoc_json_path)
}

/// Typically returns the absolute path to the regular cargo `./target`
/// directory. But also handles packages part of workspaces.
fn target_directory(manifest_path: impl AsRef<Path>) -> Result<PathBuf, BuildError> {
    let mut metadata_cmd = cargo_metadata::MetadataCommand::new();
    metadata_cmd.manifest_path(manifest_path.as_ref());
    let metadata = metadata_cmd.exec()?;
    Ok(metadata.target_directory.as_std_path().to_owned())
}

/// Figures out the name of the library crate in the current directory by
/// looking inside `Cargo.toml`
fn package_name(manifest_path: impl AsRef<Path>) -> Result<String, BuildError> {
    let manifest = cargo_toml::Manifest::from_path(&manifest_path)?;
    Ok(manifest
        .package
        .ok_or_else(|| BuildError::VirtualManifest(manifest_path.as_ref().to_owned()))?
        .name)
}
