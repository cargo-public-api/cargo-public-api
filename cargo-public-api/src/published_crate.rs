//! Creates a dummy project with a dependency on the crate we want to build
//! rustdoc JSON for. We then build rustdoc JSON for the crate using this dummy
//! project.

use crate::{Args, LATEST_VERSION_ARG};
use anyhow::{anyhow, Context, Result};
use crates_index::{Crate, Version};
use std::path::PathBuf;

pub fn build_rustdoc_json(version: Option<&str>, args: &Args) -> Result<PathBuf> {
    let package_name = package_name_from_args(args).ok_or_else(|| anyhow!("You must specify a package with either `-p package-name` or `--manifest-path path/to/Cargo.toml`"))?;
    let crate_ = http_get_crate(&package_name, args.verbose)?;
    let crate_version = get_crate_version(&crate_, version)?;
    let build_dir = build_dir(args, &crate_version);
    std::fs::create_dir_all(&build_dir)?;

    let write_file = |name: &str, contents: &str| -> std::io::Result<PathBuf> {
        let mut path = build_dir.clone();
        path.push(name);
        std::fs::write(&path, contents)?;
        Ok(path)
    };

    write_file("lib.rs", "// empty lib")?;
    let manifest = manifest_for(args, &crate_version)?;
    let manifest = write_file("Cargo.toml", &manifest)?;

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

fn get_crate_version(crate_: &Crate, version: Option<&str>) -> Result<Version, anyhow::Error> {
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
                    "Could not find version `{}` of crate `{}`",
                    version,
                    crate_.name(),
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
fn build_dir(args: &Args, crate_version: &crates_index::Version) -> PathBuf {
    let mut build_dir = if let Some(target_dir) = &args.target_dir {
        target_dir.clone()
    } else {
        dirs::cache_dir().unwrap_or_else(std::env::temp_dir)
    };

    build_dir.push("cargo-public-api");
    build_dir.push("build-root-for-published-crates");
    build_dir.push(crate_version.name());
    build_dir.push("-");
    build_dir.push(crate_version.version());
    build_dir
}

/// Creates a manifest with a dependency so we can "trick" cargo into
/// downloading the dependency for us.
fn manifest_for(args: &Args, spec: &crates_index::Version) -> Result<String> {
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

    Ok(format!(
        "{setup}\n[dependencies.{}]\n{}",
        spec.name(),
        toml::to_string(&cargo_manifest::DependencyDetail {
            version: Some(format!("={}", spec.version())),
            default_features: no_default_features.then(|| false),
            features: if *all_features {
                Some(spec.features().keys().map(Clone::clone).collect())
            } else if !features.is_empty() {
                Some(features.clone())
            } else {
                None
            },
            ..Default::default()
        })?
    ))
}

// We use curl to significantly reduce the number of needed deps compared to
// e.g. reqwest.
#[allow(clippy::similar_names)]
fn http_get_crate(name: &str, verbose: bool) -> Result<Crate> {
    let mut body: Vec<u8> = vec![];
    let dep_path = cargo_util::registry::make_dep_path(name, false /* prefix_only */);
    let url = format!("https://index.crates.io/{dep_path}");
    if verbose {
        eprintln!("Using {url:?} to get info about crate {name:?}");
    }

    let mut curl = curl::easy::Easy::new();
    curl.url(&url)?;
    curl.follow_location(true)?;

    let mut list = curl::easy::List::new();
    list.append("Accept: application/json")?;
    curl.http_headers(list)?;

    {
        let mut transfer = curl.transfer();
        transfer.write_function(|data| {
            body.extend_from_slice(data);
            Ok(data.len())
        })?;
        transfer.perform()?;
    }

    Crate::from_slice(&body).with_context(|| anyhow!("Could not find crate `{name}`"))
}
