//! Creates a dummy project with a dependency on the crate we want to build
//! rustdoc JSON for. We then build rustdoc JSON for the crate using this dummy
//! project.

use crate::{Args, LATEST_VERSION_ARG};
use anyhow::{anyhow, Result};
use std::path::PathBuf;

pub fn build_rustdoc_json(version: impl Into<String>, args: &Args) -> Result<PathBuf> {
    let package_name = package_name_from_args(args).ok_or_else(|| anyhow!("You must specify a package with either `-p package-name` or `--manifest-path path/to/Cargo.toml`"))?;

    let mut version = version.into();
    if version == LATEST_VERSION_ARG {
        version = most_recent_version_for_package(&package_name)?;
        eprintln!("Resolved `diff {LATEST_VERSION_ARG}` to `diff {version}`");
    }

    let spec = PackageSpec {
        name: package_name,
        version,
    };

    let build_dir = build_dir(args, &spec);
    std::fs::create_dir_all(&build_dir)?;

    let write_file = |name: &str, contents: &str| -> std::io::Result<PathBuf> {
        let mut path = build_dir.clone();
        path.push(name);
        std::fs::write(&path, contents)?;
        Ok(path)
    };

    write_file("lib.rs", "// empty lib")?;
    let (manifest, needs_resolved) = manifest_for(args, &spec)?;
    let manifest = write_file("Cargo.toml", &manifest)?;

    'resolve: {
        if needs_resolved {
            let mut metadata = cargo_metadata::MetadataCommand::new();
            metadata.manifest_path(&manifest);
            let metadata = metadata.exec()?;
            let Some(package) = metadata.packages.iter().find(|p| p.name == spec.name) else {
                // XXX if metadata doesn't find a package, rustdoc will tell us why, so continue
                break 'resolve;
            };

            write_file(
                "Cargo.toml",
                &manifest_for_resolved(args, &spec, Some(package))?.0,
            )?;
        }
    }

    // Since we used `crate::builder_from_args(args)` above it means that if
    // `args.target_dir` is set, both the dummy crate and the real crate will
    // write to the same JSON path since they have the same project name! That
    // won't work. So always clear the target dir before we use the builder.
    let builder = crate::api_source::builder_from_args(args)
        .clear_target_dir()
        .all_features(false)
        .features(Vec::<&str>::new())
        .no_default_features(false)
        .manifest_path(&manifest)
        .package(&spec.name);
    crate::api_source::build_rustdoc_json(builder)
}

/// Gets the most recent version for the given package, by querying the
/// crates.io index that users have locally.
fn most_recent_version_for_package(package_name: &str) -> Result<String> {
    #[cfg(feature = "diff-latest")]
    {
        let index = crates_index::Index::new_cargo_default()?;
        let crate_ = index.crate_(package_name).ok_or_else(|| {
            anyhow!("Could not find crate `{package_name}` in the crates.io index")
        })?;
        let version = crate_.most_recent_version();
        Ok(version.version().to_string())
    }
    #[cfg(not(feature = "diff-latest"))]
    {
        Err(anyhow!(
            "Can not find latest version of `{package_name}`; the `diff-latest` feature needs to be enabled for `cargo-public-api`"
        ))
    }
}

/// Returns the package name from `-p package-name` or from inside
/// `--manifest-path Cargo.toml`.
fn package_name_from_args(args: &Args) -> Option<String> {
    if let Some(package) = &args.package {
        Some(package.clone())
    } else {
        let manifest = cargo_manifest::Manifest::from_path(args.manifest_path.as_path()).ok()?;
        manifest.package.map(|p| p.name)
    }
}

/// For users we prefer a non-temporary dir so repeated builds can be
/// incremental. But when tests run, they will set `args.target_dir` to a
/// temporary dir so that tests can run in parallel without interference.
fn build_dir(args: &Args, spec: &PackageSpec) -> PathBuf {
    let mut build_dir = if let Some(target_dir) = &args.target_dir {
        target_dir.clone()
    } else {
        dirs::cache_dir().unwrap_or_else(std::env::temp_dir)
    };

    build_dir.push("cargo-public-api");
    build_dir.push("build-root-for-published-crates");
    build_dir.push(spec.as_dir_name());
    build_dir
}
/// Create the manifest for a package given cargo cli arguments.
///
/// If the boolean is `true`, call `manifest_for_resolved`
fn manifest_for(args: &Args, spec: &PackageSpec) -> Result<(String, bool)> {
    manifest_for_resolved(args, spec, None)
}

fn manifest_for_resolved(
    args: &Args,
    spec: &PackageSpec,
    package: Option<&cargo_metadata::Package>,
) -> Result<(String, bool)> {
    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn is_true(b: &bool) -> bool {
        *b
    }

    #[derive(serde::Serialize)]
    struct Dep<'a, S: AsRef<str>> {
        version: &'a str,
        #[serde(skip_serializing_if = "is_true")]
        default_features: bool,
        #[serde(skip_serializing_if = "<[_]>::is_empty")]
        features: &'a [S],
    }

    let setup = toml::toml! {
        [package]
        name = "crate-downloader"
        version = "0.1.0"
        edition = "2021"
        [lib]
        path = "lib.rs"
    };

    let mut needs_resolved = false;

    let dep = match (args, package) {
        (
            Args {
                features,
                all_features: false,
                no_default_features,
                ..
            },
            _,
        ) => {
            format!(
                "[dependencies.{}]\n{}",
                spec.name,
                toml::to_string(&Dep {
                    version: &spec.version,
                    default_features: !no_default_features,
                    features
                })?
            )
        }
        (
            Args {
                all_features: true, ..
            },
            None,
        ) => {
            needs_resolved = true;
            format!(
                "[dependencies.{}]\n{}",
                spec.name,
                toml::to_string(&Dep {
                    version: &spec.version,
                    default_features: true,
                    features: &Vec::<&str>::new()
                })?
            )
        }
        (
            Args {
                all_features: true, ..
            },
            Some(package),
        ) => {
            needs_resolved = true;
            format!(
                "[dependencies.{}]\n{}",
                spec.name,
                toml::to_string(&Dep {
                    version: &spec.version,
                    default_features: true,
                    features: &package.features.keys().collect::<Vec<_>>(),
                })?
            )
        }
    };

    Ok((format!("{setup}\n{dep}"), needs_resolved))
}

#[derive(Debug, PartialEq, Eq)]
struct PackageSpec {
    name: String,
    version: String,
}

impl PackageSpec {
    fn as_dir_name(&self) -> PathBuf {
        PathBuf::from(format!("{}-{}", self.name, self.version))
    }
}
