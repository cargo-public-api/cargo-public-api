use super::BuildError;

use std::{
    path::{Path, PathBuf},
    process::Command,
};

/// For development purposes only. Sometimes when you work on this project you
/// want to quickly use a different toolchain to build rustdoc JSON. You can
/// specify what toolchain, by temporarily changing this.
const OVERRIDDEN_TOOLCHAIN: Option<&str> = option_env!("RUSTDOC_JSON_OVERRIDDEN_TOOLCHAIN_HACK"); // Some("nightly-2022-07-16");

/// Run `cargo rustdoc` to produce rustdoc JSON and return the path to the built
/// file.
pub fn run_cargo_rustdoc(options: Builder) -> Result<PathBuf, BuildError> {
    let mut cmd = cargo_rustdoc_command(&options);
    if cmd.status()?.success() {
        rustdoc_json_path_for_manifest_path(
            options.manifest_path,
            options.package.as_deref(),
            &options.package_target,
            options.target_dir.as_deref(),
            options.target.as_deref(),
        )
    } else {
        let manifest = cargo_manifest::Manifest::from_path(&options.manifest_path)?;
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
fn cargo_rustdoc_command(options: &Builder) -> Command {
    let Builder {
        toolchain: requested_toolchain,
        manifest_path,
        target_dir,
        target,
        quiet,
        no_default_features,
        all_features,
        features,
        package,
        package_target,
        document_private_items,
        cap_lints,
    } = options;

    let mut command = OVERRIDDEN_TOOLCHAIN
        .or(requested_toolchain.as_deref())
        .map_or_else(
            || Command::new("cargo"),
            |toolchain| {
                let mut cmd = Command::new("rustup");
                cmd.args(["run", toolchain, "cargo"]);
                cmd
            },
        );

    command.arg("rustdoc");
    match package_target {
        PackageTarget::Lib => command.arg("--lib"),
        PackageTarget::Bin(target) => command.args(["--bin", target]),
        PackageTarget::Example(target) => command.args(["--example", target]),
        PackageTarget::Test(target) => command.args(["--test", target]),
        PackageTarget::Bench(target) => command.args(["--bench", target]),
    };
    if let Some(target_dir) = target_dir {
        command.arg("--target-dir");
        command.arg(target_dir);
    }
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
    if *document_private_items {
        command.arg("--document-private-items");
    }
    if let Some(cap_lints) = cap_lints {
        command.args(["--cap-lints", cap_lints]);
    }
    command
}

/// Returns `./target/doc/crate_name.json`. Also takes care of transforming
/// `crate-name` to `crate_name`.
fn rustdoc_json_path_for_manifest_path(
    manifest_path: impl AsRef<Path>,
    package: Option<&str>,
    package_target: &PackageTarget,
    target_dir: Option<&Path>,
    target: Option<&str>,
) -> Result<PathBuf, BuildError> {
    let target_dir = match target_dir {
        Some(target_dir) => target_dir.to_owned(),
        None => target_directory(&manifest_path)?,
    };

    // get the name of the crate/binary/example/test/bench
    let package_target_name = match package_target {
        PackageTarget::Lib => package
            .map(ToOwned::to_owned)
            .map_or_else(|| package_name(&manifest_path), Ok)?,
        PackageTarget::Bin(package)
        | PackageTarget::Example(package)
        | PackageTarget::Test(package)
        | PackageTarget::Bench(package) => package.clone(),
    };

    let mut rustdoc_json_path = target_dir;
    // if one has specified a target explicitly then Cargo appends that target triple name as a subfolder
    if let Some(target) = target {
        rustdoc_json_path.push(target);
    }
    rustdoc_json_path.push("doc");
    rustdoc_json_path.push(package_target_name.replace('-', "_"));
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
    let manifest = cargo_manifest::Manifest::from_path(manifest_path.as_ref())?;
    Ok(manifest
        .package
        .ok_or_else(|| BuildError::VirtualManifest(manifest_path.as_ref().to_owned()))?
        .name)
}

/// Builds rustdoc JSON. There are many build options. Refer to the docs to
/// learn about them all. See [top-level docs](crate) for an example on how to use this builder.
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Debug)]
pub struct Builder {
    toolchain: Option<String>,
    manifest_path: PathBuf,
    target_dir: Option<PathBuf>,
    target: Option<String>,
    quiet: bool,
    no_default_features: bool,
    all_features: bool,
    features: Vec<String>,
    package: Option<String>,
    package_target: PackageTarget,
    document_private_items: bool,
    cap_lints: Option<String>,
}

impl Default for Builder {
    fn default() -> Self {
        Self {
            toolchain: None,
            manifest_path: PathBuf::from("Cargo.toml"),
            target_dir: None,
            target: None,
            quiet: false,
            no_default_features: false,
            all_features: false,
            features: vec![],
            package: None,
            package_target: PackageTarget::Lib,
            document_private_items: false,
            cap_lints: Some(String::from("warn")),
        }
    }
}

impl Builder {
    /// Set the toolchain. Default: `None`.
    /// Until rustdoc JSON has stabilized, you will want to set this to
    /// be `"nightly"` or similar.
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

    /// Set what `--target-dir` to pass to `cargo`. Typically only needed if you
    /// want to be able to build rustdoc JSON for the same crate concurrently,
    /// for example to parallelize regression tests.
    #[must_use]
    pub fn target_dir(mut self, target_dir: impl AsRef<Path>) -> Self {
        self.target_dir = Some(target_dir.as_ref().to_owned());
        self
    }

    /// Clear a target dir previously set with [`Self::target_dir`].
    #[must_use]
    pub fn clear_target_dir(mut self) -> Self {
        self.target_dir = None;
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

    /// What part of the package to document. Default: `PackageTarget::Lib`
    #[must_use]
    pub fn package_target<T: AsRef<str>>(mut self, package_target: PackageTarget<T>) -> Self {
        self.package_target = match package_target {
            PackageTarget::Lib => PackageTarget::Lib,
            PackageTarget::Bin(target) => PackageTarget::Bin(target.as_ref().to_string()),
            PackageTarget::Example(target) => PackageTarget::Example(target.as_ref().to_string()),
            PackageTarget::Test(target) => PackageTarget::Test(target.as_ref().to_string()),
            PackageTarget::Bench(target) => PackageTarget::Bench(target.as_ref().to_string()),
        };
        self
    }

    /// Whether to pass `--document-private-items` to `cargo rustdoc`. Default: `false`
    #[must_use]
    pub fn document_private_items(mut self, document_private_items: bool) -> Self {
        self.document_private_items = document_private_items;
        self
    }

    /// What to pass as `--cap-lints` to rustdoc JSON build command
    #[must_use]
    pub fn cap_lints(mut self, cap_lints: Option<impl AsRef<str>>) -> Self {
        self.cap_lints = cap_lints.map(|c| c.as_ref().to_owned());
        self
    }

    /// Generate rustdoc JSON for a library crate. Returns the path to the freshly
    /// built rustdoc JSON file.
    ///
    /// See [top-level docs](crate) for an example on how to use it.
    ///
    /// # Errors
    ///
    /// E.g. if building the JSON fails or if the manifest path does not exist or is
    /// invalid.
    pub fn build(self) -> Result<PathBuf, BuildError> {
        run_cargo_rustdoc(self)
    }
}

/// The part of of the package to document
#[derive(Default, Debug, Clone)]
pub enum PackageTarget<T: AsRef<str> = String> {
    /// Document the package as a library, i.e. pass `--lib`
    #[default]
    Lib,
    /// Document the given binary, i.e. pass `--bin <name>`
    Bin(T),
    /// Document the given binary, i.e. pass `--example <name>`
    Example(T),
    /// Document the given binary, i.e. pass `--test <name>`
    Test(T),
    /// Document the given binary, i.e. pass `--bench <name>`
    Bench(T),
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
