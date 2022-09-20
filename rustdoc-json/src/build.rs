use super::BuildError;
use super::BuildOptions;

use std::{
    path::{Path, PathBuf},
    process::Command,
};

/// For development purposes only. Sometimes when you work on this project you
/// want to quickly use a different toolchain to build rustdoc JSON. You can
/// specify what toolchain, by temporarily changing this.
const OVERRIDDEN_TOOLCHAIN: Option<&str> = option_env!("RUSTDOC_JSON_OVERRIDDEN_TOOLCHAIN_HACK"); // Some("+nightly-2022-07-16");

/// Run `cargo rustdoc` to produce rustdoc JSON and return the path to the built
/// file.
pub fn run_cargo_rustdoc(options: BuildOptions) -> Result<PathBuf, BuildError> {
    let mut cmd = cargo_rustdoc_command(&options);
    if cmd.status()?.success() {
        rustdoc_json_path_for_manifest_path(
            options.manifest_path,
            options.package.as_deref(),
            options.target.as_deref(),
        )
    } else {
        let manifest = cargo_toml::Manifest::from_path(&options.manifest_path)?;
        if manifest.package.is_none() && manifest.workspace.is_some() {
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
fn cargo_rustdoc_command(options: &BuildOptions) -> Command {
    let BuildOptions {
        toolchain: requested_toolchain,
        manifest_path,
        target,
        quiet,
        no_default_features,
        all_features,
        features,
        package,
    } = options;

    let mut command = OVERRIDDEN_TOOLCHAIN
        .or(requested_toolchain.as_deref())
        .map_or_else(
            || Command::new("cargo"),
            |toolchain| {
                let mut cmd = Command::new("rustup");
                cmd.args(["run", toolchain.trim_start_matches('+'), "cargo"]);
                cmd
            },
        );

    command.arg("rustdoc");
    command.arg("--lib");
    if *quiet {
        command.arg("--quiet");
    }
    command.arg("--manifest-path");
    command.arg(manifest_path);
    if let Some(target) = target {
        command.arg("--target");
        command.arg(target);
    }
    if *no_default_features {
        command.arg("--no-default-features");
    }
    if *all_features {
        command.arg("--all-features");
    }
    for feature in features {
        command.args(["--features", feature]);
    }
    if let Some(package) = package {
        command.args(["--package", package]);
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
    package: Option<&str>,
    target: Option<&str>,
) -> Result<PathBuf, BuildError> {
    let target_dir = target_directory(&manifest_path)?;
    let lib_name = package
        .map(ToOwned::to_owned)
        .map_or_else(|| package_name(&manifest_path), Ok)?;

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
            no_default_features: false,
            all_features: false,
            features: vec![],
            package: None,
        }
    }
}

impl BuildOptions {
    /// Set the toolchain. Default: `None`.
    /// Until rustdoc JSON has stabilized, you will want to set this to
    /// be `"+nightly"` or similar.
    ///
    /// If the toolchain is set as `None`, the current active toolchain will be used.
    ///
    /// # Notes
    ///
    /// The currently active toolchain is typically specified by the
    /// `RUSTUP_TOOLCHAIN` environment variable, which the rustup proxy
    /// mechanism sets. See <https://rust-lang.github.io/rustup/overrides.html>
    /// for more info on how the active toolchain is determined.
    #[must_use]
    pub fn toolchain(mut self, toolchain: impl Into<Option<String>>) -> Self {
        self.toolchain = toolchain.into();
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
    pub const fn quiet(mut self, quiet: bool) -> Self {
        self.quiet = quiet;
        self
    }

    /// Whether or not to pass `--target` to `cargo rustdoc`. Default: `None`
    #[must_use]
    pub fn target(mut self, target: String) -> Self {
        self.target = Some(target);
        self
    }

    /// Whether to pass `--no-default-features` to `cargo rustdoc`. Default: `false`
    #[must_use]
    pub const fn no_default_features(mut self, no_default_features: bool) -> Self {
        self.no_default_features = no_default_features;
        self
    }

    /// Whether to pass `--all-features` to `cargo rustdoc`. Default: `false`
    #[must_use]
    pub const fn all_features(mut self, all_features: bool) -> Self {
        self.all_features = all_features;
        self
    }

    /// Features to pass to `cargo rustdoc` via `--features`. Default to an empty vector
    #[must_use]
    pub fn features<I: IntoIterator<Item = S>, S: AsRef<str>>(mut self, features: I) -> Self {
        self.features = features
            .into_iter()
            .map(|item| item.as_ref().to_owned())
            .collect();
        self
    }

    /// Package to use for `cargo rustdoc` via `-p`. Default: `None`
    #[must_use]
    pub fn package(mut self, package: impl AsRef<str>) -> Self {
        self.package = Some(package.as_ref().to_owned());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ensure_toolchain_not_overridden() {
        // The override is only meant to be changed locally, do not git commit!
        // If the var is set from the env var, that's OK, so skip the check in
        // that case.
        if option_env!("RUSTDOC_JSON_OVERRIDDEN_TOOLCHAIN_HACK").is_none() {
            assert!(OVERRIDDEN_TOOLCHAIN.is_none());
        }
    }
}
