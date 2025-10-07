use super::BuildError;
use cargo_metadata::TargetKind;
use tracing::*;

use std::collections::HashMap;
use std::ffi::{OsStr, OsString};
use std::io::Write;
use std::{
    path::{Path, PathBuf},
    process::Command,
};

/// For development purposes only. Sometimes when you work on this project you
/// want to quickly use a different toolchain to build rustdoc JSON. You can
/// specify what toolchain, by temporarily changing this.
const OVERRIDDEN_TOOLCHAIN: Option<&str> = option_env!("RUSTDOC_JSON_OVERRIDDEN_TOOLCHAIN_HACK"); // Some("nightly-2022-07-16");

struct CaptureOutput<O, E> {
    stdout: O,
    stderr: E,
}

fn run_cargo_rustdoc<O, E>(
    options: Builder,
    capture_output: Option<CaptureOutput<O, E>>,
) -> Result<PathBuf, BuildError>
where
    O: Write,
    E: Write,
{
    let mut cmd = cargo_rustdoc_command(&options)?;
    info!("Running {cmd:?}");

    let status = match capture_output {
        Some(CaptureOutput {
            mut stdout,
            mut stderr,
        }) => {
            let output = cmd.output().map_err(|e| {
                BuildError::CommandExecutionError(format!("Failed to run `{cmd:?}`: {e}"))
            })?;
            stdout.write_all(&output.stdout).map_err(|e| {
                BuildError::CapturedOutputError(format!("Failed to write stdout: {e}"))
            })?;
            stderr.write_all(&output.stderr).map_err(|e| {
                BuildError::CapturedOutputError(format!("Failed to write stderr: {e}"))
            })?;
            output.status
        }
        None => cmd.status().map_err(|e| {
            BuildError::CommandExecutionError(format!("Failed to run `{cmd:?}`: {e}"))
        })?,
    };

    if status.success() {
        rustdoc_json_path_for_manifest_path(
            &options.manifest_path,
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
            Err(BuildError::BuildRustdocJsonError)
        }
    }
}

/// Construct the `cargo rustdoc` command to use for building rustdoc JSON. The
/// command typically ends up looks something like this:
/// ```bash
/// cargo +nightly rustdoc --lib --manifest-path Cargo.toml -- -Z unstable-options --output-format json --cap-lints warn
/// ```
fn cargo_rustdoc_command(options: &Builder) -> Result<Command, BuildError> {
    let Builder {
        toolchain: requested_toolchain,
        manifest_path,
        target_dir,
        target,
        quiet,
        silent,
        color,
        no_default_features,
        all_features,
        features,
        package,
        package_target,
        document_private_items,
        cap_lints,
        envs,
    } = options;

    let mut command = match OVERRIDDEN_TOOLCHAIN.or(requested_toolchain.as_deref()) {
        None => Command::new("cargo"),
        Some(toolchain) => {
            if !rustup_installed() {
                return Err(BuildError::General(String::from(
                    "required program rustup not found in PATH. Is it installed?",
                )));
            }
            let mut cmd = Command::new("rustup");
            cmd.args(["run", toolchain, "cargo"]);
            cmd
        }
    };

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
    if *silent {
        command.stdout(std::process::Stdio::null());
        command.stderr(std::process::Stdio::null());
    }
    match *color {
        Color::Always => command.arg("--color").arg("always"),
        Color::Never => command.arg("--color").arg("never"),
        Color::Auto => command.arg("--color").arg("auto"),
    };
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
    command.envs(envs);
    Ok(command)
}

/// Returns `./target/doc/crate_name.json`. Also takes care of transforming
/// `crate-name` to `crate_name`. Also handles `[lib] name = "foo"`.
#[instrument(ret(level = Level::DEBUG))]
fn rustdoc_json_path_for_manifest_path(
    manifest_path: &Path,
    package: Option<&str>,
    package_target: &PackageTarget,
    target_dir: Option<&Path>,
    target: Option<&str>,
) -> Result<PathBuf, BuildError> {
    let target_dir = match target_dir {
        Some(target_dir) => target_dir.to_owned(),
        None => target_directory(manifest_path)?,
    };

    // get the name of the crate/binary/example/test/bench
    let package_target_name = match package_target {
        PackageTarget::Lib => library_name(manifest_path, package)?,
        PackageTarget::Bin(name)
        | PackageTarget::Example(name)
        | PackageTarget::Test(name)
        | PackageTarget::Bench(name) => name.clone(),
    }
    .replace('-', "_");

    let mut rustdoc_json_path = target_dir;
    // if one has specified a target explicitly then Cargo appends that target triple name as a subfolder
    if let Some(target) = target {
        rustdoc_json_path.push(target);
    }
    rustdoc_json_path.push("doc");
    rustdoc_json_path.push(package_target_name);
    rustdoc_json_path.set_extension("json");
    Ok(rustdoc_json_path)
}

/// Checks if the `rustup` program can be found in `PATH`.
pub fn rustup_installed() -> bool {
    let mut check_rustup = std::process::Command::new("rustup");
    check_rustup.arg("--version");
    check_rustup.stdout(std::process::Stdio::null());
    check_rustup.stderr(std::process::Stdio::null());
    check_rustup.status().map(|s| s.success()).unwrap_or(false)
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
/// `Cargo.toml` and `package_name` (in case Cargo.toml is a workspace root).
fn library_name(
    manifest_path: impl AsRef<Path>,
    package_name: Option<&str>,
) -> Result<String, BuildError> {
    let package_name = if let Some(package_name) = package_name {
        package_name.to_owned()
    } else {
        // We must figure out the package name ourselves from the manifest.
        let manifest = cargo_manifest::Manifest::from_path(manifest_path.as_ref())?;
        manifest
            .package
            .ok_or_else(|| BuildError::VirtualManifest(manifest_path.as_ref().to_owned()))?
            .name
            .to_owned()
    };

    let mut metadata_cmd = cargo_metadata::MetadataCommand::new();
    metadata_cmd.manifest_path(manifest_path.as_ref());
    let metadata = metadata_cmd.exec()?;

    let package = metadata
        .packages
        .into_iter()
        .find(|p| p.name.as_str() == package_name)
        .ok_or_else(|| BuildError::VirtualManifest(manifest_path.as_ref().to_owned()))?;

    for target in &package.targets {
        if target.kind.contains(&TargetKind::Lib) {
            return Ok(target.name.to_owned());
        }
    }

    Ok(package.name.into_inner())
}

/// Color configuration for the output of `cargo rustdoc`.
#[derive(Clone, Copy, Debug)]
pub enum Color {
    /// Always output colors.
    Always,
    /// Never output colors.
    Never,
    /// Cargo will decide whether to output colors based on the tty type.
    Auto,
}

/// Builds rustdoc JSON. There are many build options. Refer to the docs to
/// learn about them all. See [top-level docs](crate) for an example on how to use this builder.
#[derive(Clone, Debug)]
pub struct Builder {
    toolchain: Option<String>,
    manifest_path: PathBuf,
    target_dir: Option<PathBuf>,
    target: Option<String>,
    quiet: bool,
    silent: bool,
    color: Color,
    no_default_features: bool,
    all_features: bool,
    features: Vec<String>,
    package: Option<String>,
    package_target: PackageTarget,
    document_private_items: bool,
    cap_lints: Option<String>,
    envs: HashMap<OsString, OsString>,
}

impl Default for Builder {
    fn default() -> Self {
        Self {
            toolchain: None,
            manifest_path: PathBuf::from("Cargo.toml"),
            target_dir: None,
            target: None,
            quiet: false,
            silent: false,
            color: Color::Auto,
            no_default_features: false,
            all_features: false,
            features: vec![],
            package: None,
            package_target: PackageTarget::default(),
            document_private_items: false,
            cap_lints: Some(String::from("warn")),
            envs: HashMap::new(),
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
    pub fn toolchain(mut self, toolchain: impl Into<String>) -> Self {
        self.toolchain = Some(toolchain.into());
        self
    }

    /// Clear a toolchain previously set with [`Self::toolchain`].
    #[must_use]
    pub fn clear_toolchain(mut self) -> Self {
        self.toolchain = None;
        self
    }

    /// Set the relative or absolute path to `Cargo.toml`. Default: `Cargo.toml`
    #[must_use]
    pub fn manifest_path(mut self, manifest_path: impl AsRef<Path>) -> Self {
        manifest_path.as_ref().clone_into(&mut self.manifest_path);
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

    /// Whether or not to redirect stdout and stderr to /dev/null. Default: `false`
    #[must_use]
    pub const fn silent(mut self, silent: bool) -> Self {
        self.silent = silent;
        self
    }

    /// Color configuration for the output of `cargo rustdoc`.
    #[must_use]
    pub const fn color(mut self, color: Color) -> Self {
        self.color = color;
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
    pub fn package_target(mut self, package_target: PackageTarget) -> Self {
        self.package_target = package_target;
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

    /// Environment variable mapping to pass to the spawned `cargo rustdoc` process.
    ///
    /// # Notes
    ///
    /// - Environment variable names are case-insensitive (but case-preserving) on Windows and case-sensitive on all other platforms.
    /// - Spawned `cargo rustdoc` processes will inherit environment variables from their parent process by default.
    ///   Environment variables explicitly set using this take precedence over inherited variables.
    #[must_use]
    pub fn env(mut self, key: impl AsRef<OsStr>, val: impl AsRef<OsStr>) -> Self {
        self.envs
            .insert(key.as_ref().to_owned(), val.as_ref().to_owned());
        self
    }

    /// Generate rustdoc JSON for a crate. Returns the path to the freshly
    /// built rustdoc JSON file.
    ///
    /// This method will print the stdout and stderr of the `cargo rustdoc` command to the stdout
    /// and stderr of the calling process. If you want to capture the output, use
    /// [`Builder::build_with_captured_output()`].
    ///
    /// See [top-level docs](crate) for an example on how to use it.
    ///
    /// # Errors
    ///
    /// E.g. if building the JSON fails or if the manifest path does not exist or is
    /// invalid.
    pub fn build(self) -> Result<PathBuf, BuildError> {
        run_cargo_rustdoc::<std::io::Sink, std::io::Sink>(self, None)
    }

    /// Generate rustdoc JSON for a crate. This works like [`Builder::build()`], but will
    /// capture the stdout and stderr of the `cargo rustdoc` command. The output will be written to
    /// the `stdout` and `stderr` parameters. In particular, potential warnings and errors emitted
    /// by `cargo rustdoc` will be captured to `stderr`. This can be useful if you want to present
    /// these errors to the user only when the build failed. Here's an example of how that might
    /// look like:
    ///
    /// ```no_run
    /// # use std::path::PathBuf;
    /// # use rustdoc_json::BuildError;
    /// #
    /// let mut stderr: Vec<u8> = Vec::new();
    ///
    /// let result: Result<PathBuf, BuildError> = rustdoc_json::Builder::default()
    ///     .toolchain("nightly")
    ///     .manifest_path("Cargo.toml")
    ///     .build_with_captured_output(std::io::sink(), &mut stderr);
    ///
    /// match result {
    ///     Err(BuildError::BuildRustdocJsonError) => {
    ///         eprintln!("Crate failed to build:\n{}", String::from_utf8_lossy(&stderr));
    ///     }
    ///     Err(e) => {
    ///        eprintln!("Error generating the rustdoc json: {}", e);
    ///     }
    ///     Ok(json_path) => {
    ///         // Do something with the json_path.
    ///     }
    /// }
    /// ```
    pub fn build_with_captured_output(
        self,
        stdout: impl Write,
        stderr: impl Write,
    ) -> Result<PathBuf, BuildError> {
        let capture_output = CaptureOutput { stdout, stderr };
        run_cargo_rustdoc(self, Some(capture_output))
    }
}

/// The part of the package to document
#[derive(Default, Debug, Clone)]
#[non_exhaustive]
pub enum PackageTarget {
    /// Document the package as a library, i.e. pass `--lib`
    #[default]
    Lib,
    /// Document the given binary, i.e. pass `--bin <name>`
    Bin(String),
    /// Document the given binary, i.e. pass `--example <name>`
    Example(String),
    /// Document the given binary, i.e. pass `--test <name>`
    Test(String),
    /// Document the given binary, i.e. pass `--bench <name>`
    Bench(String),
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
