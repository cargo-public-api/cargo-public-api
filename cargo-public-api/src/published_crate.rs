//! Creates a dummy project with a dependency on the crate we want to build
//! rustdoc JSON for. We then build rustdoc JSON for the crate using this dummy
//! project.

use crate::Args;
use anyhow::{anyhow, Result};
use std::{fmt::Display, path::PathBuf};

pub fn build_rustdoc_json(package_spec_str: &str, args: &Args) -> Result<PathBuf> {
    let spec = PackageSpec::try_from(package_spec_str)?;

    let build_dir = build_dir(&spec);
    std::fs::create_dir_all(&build_dir)?;

    let write_file = |name: &str, contents: &str| -> std::io::Result<PathBuf> {
        let mut path = build_dir.clone();
        path.push(name);
        std::fs::write(&path, contents)?;
        Ok(path)
    };

    write_file("lib.rs", "// empty lib")?;
    let manifest = write_file("Cargo.toml", &manifest_for(&spec))?;

    let builder = crate::builder_from_args(args)
        .manifest_path(&manifest)
        .package(&spec.name);
    crate::build_rustdoc_json(builder)
}

/// Prefer a non-temporary dir so repeated builds can be incremental.
fn build_dir(spec: &PackageSpec) -> PathBuf {
    let mut build_dir = dirs::cache_dir().unwrap_or_else(std::env::temp_dir);
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
}

impl Display for PackageSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}@{}", &self.name, &self.version)
    }
}

impl TryFrom<&str> for PackageSpec {
    type Error = anyhow::Error;

    fn try_from(spec_str: &str) -> Result<Self, Self::Error> {
        let mut split = spec_str.split('@');
        let name = split.next().map(str::to_owned);
        let version = split.next().map(str::to_owned);

        match (name, version) {
            (Some(name), Some(version)) if !name.is_empty() && !version.is_empty() => Ok(Self {
                name,
                version,
            }),
            _ => Err(anyhow!("Invalid format of package spec string. Use `crate-name@version`, e.g. `rustdoc-json@0.4.0`")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_spec() {
        assert!(PackageSpec::try_from("").is_err());
        assert!(PackageSpec::try_from("@").is_err());
        assert!(PackageSpec::try_from("foo@").is_err());
        assert!(PackageSpec::try_from("@1.0.0").is_err());
        assert_eq!(
            PackageSpec::try_from("foo@1.0.0").unwrap(),
            PackageSpec {
                name: String::from("foo"),
                version: String::from("1.0.0")
            }
        );
    }
}
