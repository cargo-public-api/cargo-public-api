//! Contains various ways of obtaining the public API for crates.

use anyhow::{anyhow, Context, Result};
use rustdoc_json::BuildError;
use std::path::{Path, PathBuf};

use public_api::{Options, PublicApi, MINIMUM_NIGHTLY_VERSION};

use crate::{git_utils, Args};

/// Represents some place from which a public API can be obtained.
/// Examples: a published crate, a git commit, an existing file.
pub trait ApiSource {
    /// Do the work necessary to obtain the public API.
    fn obtain_api(&self, args: &Args) -> Result<PublicApi>;

    /// If this source modifies the local git repo. If that is the case, whoever
    /// uses this API source must make sure to restore the git repo to the
    /// original state afterwards. The API source itself does not do any
    /// restoration.
    fn changes_commit(&self) -> bool {
        false
    }

    fn boxed(self) -> Box<dyn ApiSource>
    where
        Self: Sized + 'static,
    {
        Box::new(self)
    }
}

/// The API is obtained by building the crate in the current directory.
pub struct CurrentDir;

impl ApiSource for CurrentDir {
    fn obtain_api(&self, args: &Args) -> Result<PublicApi> {
        public_api_for_current_dir(args)
    }
}
/// The API is obtained from a crate published to crates.io. This struct only
/// contains the version. The name of the package is obtained via [`Args`].
/// Either via `-p` or via `--manifest-path`.
pub struct PublishedCrate {
    version: String,
}

impl PublishedCrate {
    pub fn new(version: &str) -> Self {
        Self {
            version: version.to_owned(),
        }
    }
}

impl ApiSource for PublishedCrate {
    fn obtain_api(&self, args: &Args) -> Result<public_api::PublicApi> {
        let rustdoc_json = crate::published_crate::build_rustdoc_json(&self.version, args)?;
        public_api_from_rustdoc_json(&rustdoc_json, args)
    }
}

/// The API is obtained from a git commit.
pub struct Commit {
    commit: String,
}

impl Commit {
    pub fn new(args: &Args, commit_ref: &str) -> Result<Self> {
        Ok(Self {
            // Resolve the ref during creation to detect problems early
            commit: git_utils::resolve_ref(args.git_root()?, commit_ref)?,
        })
    }
}

impl ApiSource for Commit {
    fn obtain_api(&self, args: &Args) -> Result<PublicApi> {
        crate::git_checkout(args, &self.commit)?;
        public_api_for_current_dir(args)
    }

    fn changes_commit(&self) -> bool {
        true
    }
}

/// The API is obtained from an existing rustdoc JSON file.
pub struct RustdocJson {
    path: PathBuf,
}

impl RustdocJson {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl ApiSource for RustdocJson {
    fn obtain_api(&self, args: &Args) -> Result<PublicApi> {
        public_api_from_rustdoc_json(&self.path, args)
    }
}

/// Builds the public API for the library in the current working directory. Note
/// that we sometimes checkout a different commit before invoking this function,
/// which means it will return the public API of that commit.
fn public_api_for_current_dir(args: &Args) -> Result<PublicApi> {
    let json_path = rustdoc_json_for_current_dir(args)?;
    public_api_from_rustdoc_json(&json_path, args)
}

/// Builds the rustdoc JSON for the library in the current working directory.
/// Also see [`public_api_for_current_dir()`].
fn rustdoc_json_for_current_dir(args: &Args) -> Result<PathBuf> {
    let builder = builder_from_args(args);
    build_rustdoc_json(builder)
}

/// Helper to build rustdoc JSON with a builder while also handling any virtual
/// manifest errors.
pub fn build_rustdoc_json(builder: rustdoc_json::Builder) -> Result<PathBuf> {
    match builder.build() {
        Err(BuildError::VirtualManifest(manifest_path)) => virtual_manifest_error(&manifest_path),
        res => Ok(res?),
    }
}

/// Figure out what [`Options`] to pass to
/// [`public_api::PublicApi::from_rustdoc_json`] based on our
/// [`Args`]
fn get_options(args: &Args) -> Options {
    let mut options = Options::default();
    options.debug_sorting = args.debug_sorting;
    options.omit_blanket_impls = args.simplified();
    options.omit_auto_trait_impls = args.simplified();
    options.omit_auto_derived_impls = args.omit_auto_derived_impls();
    options
}

/// Creates a rustdoc JSON builder based on the args to this program.
pub fn builder_from_args(args: &Args) -> rustdoc_json::Builder {
    let mut builder = rustdoc_json::Builder::default()
        .manifest_path(&args.manifest_path)
        .all_features(args.all_features)
        .no_default_features(args.no_default_features)
        .features(&args.features);
    if let Some(toolchain) = &args.toolchain {
        builder = builder.toolchain(toolchain);
    }
    if let Some(target_dir) = &args.target_dir {
        builder = builder.target_dir(target_dir.clone());
    }
    if let Some(target) = &args.target {
        builder = builder.target(target.clone());
    }
    if let Some(package) = &args.package {
        builder = builder.package(package);
    }
    if let Some(cap_lints) = &args.cap_lints {
        builder = builder.cap_lints(Some(cap_lints));
    }
    builder
}

fn public_api_from_rustdoc_json(json_path: &Path, args: &Args) -> Result<PublicApi> {
    let options = get_options(args);

    if args.verbose {
        println!("Processing {json_path:?}");
    }

    let public_api = PublicApi::from_rustdoc_json(json_path, options).with_context(|| {
        format!(
            "Failed to parse rustdoc JSON at {json_path:?}.\n\
            This version of `cargo public-api` requires at least:\n\n    {MINIMUM_NIGHTLY_VERSION}\n\n\
            If you have that, it might be `cargo public-api` that is out of date. Try\n\
            to install the latest version with `cargo install cargo-public-api`. If the\n\
            issue remains, please report at\n\n    https://github.com/Enselic/cargo-public-api/issues",
        )
    })?;

    if args.verbose {
        public_api.missing_item_ids().for_each(|i| {
            println!("NOTE: rustdoc JSON missing referenced item with ID \"{i}\"");
        });
    }

    Ok(public_api)
}

fn virtual_manifest_error(manifest_path: &Path) -> Result<PathBuf> {
    Err(anyhow!(
        "`{:?}` is a virtual manifest.

Listing or diffing the public API of an entire workspace is not supported.

Try

    cargo public-api -p specific-crate
",
        manifest_path
    ))
}
