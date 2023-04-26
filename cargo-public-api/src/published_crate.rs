//! Creates a dummy project with a dependency on the crate we want to build
//! rustdoc JSON for. We then build rustdoc JSON for the crate using this dummy
//! project.

use crate::{Args, LATEST_VERSION_ARG};
use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};

pub fn build_rustdoc_json(version: Option<&str>, args: &Args) -> Result<PathBuf> {
    let package_name = package_name_from_args(args).ok_or_else(|| anyhow!("You must specify a package with either `-p package-name` or `--manifest-path path/to/Cargo.toml`"))?;

    let index = crates_index::Index::new_cargo_default()?;
    let crate_ = index.crate_(&package_name).ok_or_else(|| {
        anyhow!(
            "Could not find crate `{package_name}` in {:?}",
            index.path()
        )
    })?;

    let crate_version = get_crate_version(&crate_, version, index.path())?;
    let build_dir = build_dir(args, &crate_version);
    std::fs::create_dir_all(&build_dir)?;

    let write_file = |name: &str, contents: &str| -> std::io::Result<PathBuf> {
        let mut path = build_dir.clone();
        path.push(name);
        std::fs::write(&path, contents)?;
        Ok(path)
    };

    write_file("lib.rs", "// empty lib")?;
    let (manifest, needs_resolved_package) =
        manifest_simple(args, crate_version.name(), crate_version.version())?;
    let manifest = write_file("Cargo.toml", &manifest)?;

    if needs_resolved_package {
        let mut metadata = cargo_metadata::MetadataCommand::new();
        metadata.manifest_path(&manifest);
        let metadata = metadata.exec()?;
        if let Some(package) = metadata
            .packages
            .iter()
            .find(|p| p.name == crate_version.name())
        {
            // XXX if metadata doesn't find a package, rustdoc will tell us why, so we can continue
            write_file(
                "Cargo.toml",
                &manifest_with_info(args, crate_version.name(), crate_version.version(), package)?,
            )?;
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
        .manifest_path(manifest)
        .package(crate_version.name());
    crate::api_source::build_rustdoc_json(builder)
}

fn get_crate_version(
    crate_: &crates_index::Crate,
    version: Option<&str>,
    index_path: &Path,
) -> Result<crates_index::Version, anyhow::Error> {
    match version {
        Some(LATEST_VERSION_ARG) | None => {
            let resolved = if version.is_none() {
                "diff"
            } else {
                "diff latest"
            };
            let crate_version = crate_.highest_version().clone();
            eprintln!(
                "Resolved `{resolved}` to `diff {}`",
                crate_version.version()
            );
            Ok(crate_version)
        }
        Some(version) => crate_
            .versions()
            .iter()
            .find(|cv| cv.version() == version)
            .map(Clone::clone)
            .ok_or_else(|| {
                anyhow!(
                    "Could not find version `{}` of crate `{}` in {:?}",
                    version,
                    crate_.name(),
                    index_path,
                )
            }),
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
fn build_dir(args: &Args, spec: &crates_index::Version) -> PathBuf {
    let mut build_dir = if let Some(target_dir) = &args.target_dir {
        target_dir.clone()
    } else {
        dirs::cache_dir().unwrap_or_else(std::env::temp_dir)
    };

    build_dir.push("cargo-public-api");
    build_dir.push("build-root-for-published-crates");
    build_dir.push(spec.name());
    build_dir.push("-");
    build_dir.push(spec.version());
    build_dir
}

/// Create the manifest for a package given cargo cli arguments. Returns a boolean to signify if [`manifest_with_info`] needs to be called
fn manifest_simple(args: &Args, crate_name: &str, crate_version: &str) -> Result<(String, bool)> {
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
            crate_name,
            toml::to_string(&cargo_manifest::DependencyDetail {
                version: Some(format!("={crate_version}")),
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
    crate_name: &str,
    crate_version: &str,
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
            crate_name,
            toml::to_string(&cargo_manifest::DependencyDetail {
                version: Some(format!("={crate_version}")),
                features: if features.is_empty() {
                    None
                } else {
                    Some(features)
                },
                ..Default::default()
            })?
        ),
        _ => return Ok(manifest_simple(args, crate_name, crate_version)?.0),
    };

    Ok(format!("{setup}\n{dep}"))
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
            "example-api",
            "0.1.1",
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
            "example-api",
            "0.1.1",
            &package,
        )
        .unwrap();
        expect_file!("../tests/expected-output/manifest_with_info.txt").assert_eq(&manifest);
    }
}
