use super::BuildError;
use super::BuildOptions;

use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    process::Command,
};

/// For development purposes only. Sometimes when you work on this project you
/// want to quickly use a different toolchain to build rustdoc JSON. You can
/// specify what toolchain, by temporarily changing this.
const OVERRIDDEN_TOOLCHAIN: Option<&str> = option_env!("RUSTDOC_JSON_OVERRIDDEN_TOOLCHAIN_HACK"); // Some("+nightly-2022-07-16");

/// Run `cargo rustdoc` to produce rustdoc JSON and return the path to the built
/// file.
pub(crate) fn run_cargo_rustdoc(options: BuildOptions) -> Result<PathBuf, BuildError> {
    let status = cargo_rustdoc_command(
        options.toolchain.as_deref(),
        &options.manifest_path,
        options.target.as_deref(),
        options.quiet,
    )
    .status()?;
    if status.success() {
        rustdoc_json_path_for_manifest_path(options.manifest_path, options.target.as_deref())
    } else {
        let manifest = cargo_toml::Manifest::from_path(&options.manifest_path)?;
        if manifest.workspace.is_some() {
            Err(BuildError::VirtualManifest(options.manifest_path))
        } else {
            Err(BuildError::General(String::from("See above")))
        }
    }
}

/// Construct the `cargo rustdoc` command to use for building rustdoc JSON. The
/// command typically ends up looks something like this:
/// ```bash
/// cargo +nightly rustdoc --lib --manifest-path Cargo.toml -- -Z unstable-options --output-format json --cap-lints warn
/// ```
fn cargo_rustdoc_command(
    requested_toolchain: Option<&OsStr>,
    manifest_path: impl AsRef<Path>,
    target: Option<&str>,
    quiet: bool,
) -> Command {
    let mut command = Command::new("cargo");

    // These can override our `+nightly` with `+stable` unless we clear them
    command.env_remove("RUSTDOC");
    command.env_remove("RUSTC");

    let overridden_toolchain = OVERRIDDEN_TOOLCHAIN.map(OsStr::new);
    if let Some(toolchain) = overridden_toolchain.or(requested_toolchain) {
        command.arg(toolchain);
    }

    command.arg("rustdoc");
    command.arg("--lib");
    if quiet {
        command.arg("--quiet");
    }
    command.arg("--manifest-path");
    command.arg(manifest_path.as_ref());
    if let Some(target) = target {
        command.arg("--target");
        command.arg(target);
    }
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
    target: Option<&str>,
) -> Result<PathBuf, BuildError> {
    let target_dir = target_directory(&manifest_path)?;
    let lib_name = package_name(&manifest_path)?;

    let mut rustdoc_json_path = target_dir;
    // if one has specified a target explicitly then Cargo appends that target triple name as a subfolder
    if let Some(target) = target {
        rustdoc_json_path.push(&target);
    }
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

/// Figures out the name of the library crate corresponding to the given
/// `Cargo.toml` manifest path.
fn package_name(manifest_path: impl AsRef<Path>) -> Result<String, BuildError> {
    let manifest = cargo_toml::Manifest::from_path(&manifest_path)?;
    Ok(manifest
        .package
        .ok_or_else(|| BuildError::VirtualManifest(manifest_path.as_ref().to_owned()))?
        .name)
}

impl Default for BuildOptions {
    fn default() -> Self {
        Self {
            toolchain: None,
            manifest_path: PathBuf::from("Cargo.toml"),
            target: None,
            quiet: false,
        }
    }
}

impl BuildOptions {
    /// Set the toolchain. Default: `None`, which in practice means `"+stable"`.
    /// Until rustdoc JSON has stabilized, you will want to set this to
    /// `"+nightly"` or similar.
    #[must_use]
    pub fn toolchain(mut self, toolchain: impl AsRef<OsStr>) -> Self {
        self.toolchain = Some(toolchain.as_ref().to_owned());
        self
    }

    /// Set the relative or absolute path to `Cargo.toml`. Default: `Cargo.toml`
    #[must_use]
    pub fn manifest_path(mut self, manifest_path: impl AsRef<Path>) -> Self {
        self.manifest_path = manifest_path.as_ref().to_owned();
        self
    }

    /// Whether or not to pass `--quiet` to `cargo rustdoc`. Default: `false`
    #[must_use]
    pub fn quiet(mut self, quiet: bool) -> Self {
        self.quiet = quiet;
        self
    }

    /// Whether or not to pass `--target` to `cargo rustdoc`. Default: `None`
    #[must_use]
    pub fn target(mut self, target: String) -> Self {
        self.target = Some(target);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ensure_toolchain_not_overridden() {
        // The override is only meant to be changed locally, do not git commit!
        assert!(OVERRIDDEN_TOOLCHAIN.is_none());
    }
}
