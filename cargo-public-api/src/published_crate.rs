//! Creates a dummy project with a dependency on the crate we want to build
//! rustdoc JSON for. We then build rustdoc JSON for the crate using this dummy
//! project.

use crate::Args;
use anyhow::{anyhow, Result};
use std::{fmt::Display, path::PathBuf};

pub fn build_rustdoc_json(package_spec_str: &str, args: &Args) -> Result<PathBuf> {
    let fallback_name = package_name_from_args(args);
    let spec = PackageSpec::from_str_with_fallback(package_spec_str, fallback_name.as_deref())?;

    let build_dir = build_dir(args, &spec);
    std::fs::create_dir_all(&build_dir)?;

    let write_file = |name: &str, contents: &str| -> std::io::Result<PathBuf> {
        let mut path = build_dir.clone();
        path.push(name);
        std::fs::write(&path, contents)?;
        Ok(path)
    };

    write_file("lib.rs", "// empty lib")?;
    let manifest = write_file("Cargo.toml", &manifest_for(&spec))?;

    // Since we used `crate::builder_from_args(args)` above it means that if
    // `args.target_dir` is set, both the dummy crate and the real crate will
    // write to the same JSON path since they have the same project name! That
    // won't work. So always clear the target dir before we use the builder.
    let builder = crate::api_source::builder_from_args(args)
        .clear_target_dir()
        .manifest_path(&manifest)
        .package(&spec.name);
    crate::api_source::build_rustdoc_json(builder)
}

/// When diffing against a published crate, we want to allow the user to not
/// specify the package name. Instead, we want to support to figure that out for
/// the user. So instead of doing `diff --published crate-name@1.2.3` they can
/// just do `diff --published @1.2.3`. This helper function figures out what
/// package name to use in this case.
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

fn manifest_for(spec: &PackageSpec) -> String {
    format!(
        "\
        [package]\n\
        name = \"crate-downloader\"\n\
        version = \"0.1.0\"\n\
        edition = \"2021\"\n\
        [lib]\n\
        path = \"lib.rs\"\n\
        [dependencies]\n\
        {} = \"={}\"\n
        ",
        spec.name, spec.version
    )
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

    fn from_str_with_fallback(spec_str: &str, fallback_name: Option<&str>) -> Result<Self> {
        let mut split: Vec<_> = spec_str.split('@').map(ToOwned::to_owned).collect();
        let version = split.pop();
        let name = split.pop();

        match (name, version, fallback_name) {
            (Some(name), Some(version), _) if !name.is_empty() && !version.is_empty() => Ok(Self {
                name,
                version,
            }),
            (_, Some(version), Some(fallback_name)) if !fallback_name.is_empty() && !version.is_empty() => Ok(Self {
                name: fallback_name.to_owned(),
                version,
            }),
            _ => Err(anyhow!("Invalid format of package spec string. Use `crate-name@version`, e.g. `crate-name@0.4.0`")),
        }
    }
}

impl Display for PackageSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}@{}", &self.name, &self.version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_spec() {
        assert!(PackageSpec::from_str_with_fallback("", None).is_err());
        assert!(PackageSpec::from_str_with_fallback("", Some("fallback")).is_err());

        assert!(PackageSpec::from_str_with_fallback("@", None).is_err());
        assert!(PackageSpec::from_str_with_fallback("@", Some("fallback")).is_err());

        assert!(PackageSpec::from_str_with_fallback("foo@", None).is_err());
        assert!(PackageSpec::from_str_with_fallback("foo@", Some("fallback")).is_err());

        assert!(PackageSpec::from_str_with_fallback("@1.0.0", None).is_err());
        assert_eq!(
            PackageSpec::from_str_with_fallback("@1.0.0", Some("fallback")).unwrap(),
            PackageSpec {
                name: String::from("fallback"),
                version: String::from("1.0.0")
            }
        );

        assert!(PackageSpec::from_str_with_fallback("1.0.0", None).is_err());
        assert_eq!(
            PackageSpec::from_str_with_fallback("1.0.0", Some("fallback")).unwrap(),
            PackageSpec {
                name: String::from("fallback"),
                version: String::from("1.0.0")
            }
        );

        assert_eq!(
            PackageSpec::from_str_with_fallback("foo@1.0.0", None).unwrap(),
            PackageSpec {
                name: String::from("foo"),
                version: String::from("1.0.0")
            }
        );
        assert_eq!(
            PackageSpec::from_str_with_fallback("foo@1.0.0", Some("fallback")).unwrap(),
            PackageSpec {
                name: String::from("foo"),
                version: String::from("1.0.0")
            }
        );
    }
}
