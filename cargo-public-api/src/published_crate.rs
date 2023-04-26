//! Creates a dummy project with a dependency on the crate we want to build
//! rustdoc JSON for. We then build rustdoc JSON for the crate using this dummy
//! project.

use crate::{Args, LATEST_VERSION_ARG};
use anyhow::{anyhow, Result};
use std::path::PathBuf;

pub fn build_rustdoc_json(version: Option<&str>, args: &Args) -> Result<PathBuf> {
    let package_name = package_name_from_args(args).ok_or_else(|| anyhow!("You must specify a package with either `-p package-name` or `--manifest-path path/to/Cargo.toml`"))?;

    let version = match version {
        Some(LATEST_VERSION_ARG) | None => {
            let resolved = if version.is_none() {
                "diff"
            } else {
                "diff latest"
            };
            let version = latest_version_for_package(&package_name)?;
            eprintln!("Resolved `{resolved}` to `diff {version}`");
            version
        }
        Some(version) => version.into(),
    };

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
    let (manifest, needs_resolved_package) = manifest_simple(args, &spec)?;
    let manifest = write_file("Cargo.toml", &manifest)?;

    if needs_resolved_package {
        let mut metadata = cargo_metadata::MetadataCommand::new();
        metadata.manifest_path(&manifest);
        let metadata = metadata.exec()?;
        if let Some(package) = metadata.packages.iter().find(|p| p.name == spec.name) {
            // XXX if metadata doesn't find a package, rustdoc will tell us why, so we can continue
            write_file("Cargo.toml", &manifest_with_info(args, &spec, package)?)?;
        };
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
fn latest_version_for_package(package_name: &str) -> Result<String> {
    let index = crates_index::Index::new_cargo_default()?;
    let crate_ = index
        .crate_(package_name)
        .ok_or_else(|| anyhow!("Could not find crate `{package_name}` in the crates.io index"))?;

    let version = crate_.highest_version();
    Ok(version.version().to_string())
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

/// Create the manifest for a package given cargo cli arguments. Returns a boolean to signify if [`manifest_with_info`] needs to be called
fn manifest_simple(args: &Args, spec: &PackageSpec) -> Result<(String, bool)> {
    let setup = toml::toml! {
        [package]
        name = "crate-downloader"
        version = "0.1.0"
        edition = "2021"
        [lib]
        path = "lib.rs"
    };

    let Args {
        features,
        no_default_features,
        all_features,
        ..
    } = args;

    Ok((
        format!(
            "{setup}\n[dependencies.{}]\n{}",
            spec.name,
            toml::to_string(&cargo_manifest::DependencyDetail {
                version: Some(format!("={}", spec.version)),
                default_features: no_default_features.then(|| false),
                features: if features.is_empty() {
                    None
                } else {
                    Some(features.clone())
                },
                ..Default::default()
            })?
        ),
        *all_features,
    ))
}

fn manifest_with_info(
    args: &Args,
    spec: &PackageSpec,
    package: &cargo_metadata::Package,
) -> Result<String> {
    let setup = toml::toml! {
        [package]
        name = "crate-downloader"
        version = "0.1.0"
        edition = "2021"
        [lib]
        path = "lib.rs"
    };
    let features = package
        .features
        .keys()
        .map(Clone::clone)
        .collect::<Vec<_>>();
    let dep = match args {
        Args {
            all_features: true, ..
        } => format!(
            "[dependencies.{}]\n{}",
            spec.name,
            toml::to_string(&cargo_manifest::DependencyDetail {
                version: Some(format!("={}", spec.version)),
                features: if features.is_empty() {
                    None
                } else {
                    Some(features)
                },
                ..Default::default()
            })?
        ),
        _ => return Ok(manifest_simple(args, spec)?.0),
    };

    Ok(format!("{setup}\n{dep}"))
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

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser as _;
    use expect_test::expect_file;

    #[test]
    fn manifest_simple() {
        let manifest = super::manifest_simple(
            &Args::try_parse_from(["test", "-p", "example-api", "diff", "0.1.1"].iter()).unwrap(),
            &PackageSpec {
                name: "example-api".to_owned(),
                version: "0.1.1".to_owned(),
            },
        )
        .unwrap()
        .0;
        expect_file!("../tests/expected-output/manifest_simple.txt").assert_eq(&manifest);
    }

    #[test]
    fn manifest_with_info() {
        let package = serde_json::from_str(
            r#"{
        "name": "bin",
        "version": "0.1.0",
        "id": "bin 0.1.0 (path+file://)",
        "license": null,
        "license_file": null,
        "description": null,
        "source": null,
        "dependencies": [],
        "targets":
        [
          {
            "kind":
            [
              "lib"
            ],
            "crate_types":
            [
              "lib"
            ],
            "name": "lib",
            "src_path": "/src/lib.rs",
            "edition": "2021",
            "doc": true,
            "doctest": false,
            "test": true
          }
        ],
        "features": {},
        "manifest_path": "/Cargo.toml",
        "metadata": null,
        "publish": null,
        "authors": [],
        "categories": [],
        "keywords": [],
        "readme": null,
        "repository": null,
        "homepage": null,
        "documentation": null,
        "edition": "2021",
        "links": null,
        "default_run": null,
        "rust_version": null
      }"#,
        )
        .unwrap();
        let manifest = super::manifest_with_info(
            &Args::try_parse_from(["test", "-p", "example-api", "diff", "0.1.1"].iter()).unwrap(),
            &PackageSpec {
                name: "example-api".to_owned(),
                version: "0.1.1".to_owned(),
            },
            &package,
        )
        .unwrap();
        expect_file!("../tests/expected-output/manifest_with_info.txt").assert_eq(&manifest);
    }
}
