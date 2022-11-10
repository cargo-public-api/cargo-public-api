use std::path::Path;

use crate::BuildError;

#[derive(serde::Deserialize)]
pub struct Manifest {
    pub package: Option<Package>,
    pub workspace: Option<Workspace>,
}

#[derive(serde::Deserialize)]
pub struct Package {
    pub name: String,
}

#[derive(serde::Deserialize)]
pub struct Workspace {}

impl Manifest {
    pub fn from_path(manifest_path: &Path) -> Result<Self, BuildError> {
        let manifest_contents = std::fs::read_to_string(manifest_path)?;

        toml::from_str(&manifest_contents).map_err(|err| {
            BuildError::General(format!(
                "failed to deserialize package from '{}': {err}",
                manifest_path.display()
            ))
        })
    }
}
