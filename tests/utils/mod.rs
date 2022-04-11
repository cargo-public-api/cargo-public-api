use std::path::{Path, PathBuf};

/// Helper to get the path to a freshly built rustdoc JSON file for the given
/// test-crate.
pub fn rustdoc_json_path_for_crate(test_crate: &str) -> PathBuf {
    build_rustdoc_json(format!("{}/Cargo.toml", test_crate))
}

/// Helper to get a String of freshly built rustdoc JSON for the given
/// test-crate.
#[allow(unused)] // It IS used
pub fn rustdoc_json_str_for_crate(test_crate: &str) -> String {
    std::fs::read_to_string(rustdoc_json_path_for_crate(test_crate)).unwrap()
}

/// Synchronously generate the rustdoc JSON for a library crate. Returns the
/// path to the freshly built rustdoc JSON file.
fn build_rustdoc_json<P: AsRef<Path>>(manifest_path: P) -> PathBuf {
    // Synchronously invoke cargo doc
    let mut command = std::process::Command::new("cargo");
    command.args(["+nightly", "doc", "--lib", "--no-deps"]);
    command.arg("--manifest-path");
    command.arg(manifest_path.as_ref());
    command.env("RUSTDOCFLAGS", "-Z unstable-options --output-format json");
    assert!(command.spawn().unwrap().wait().unwrap().success());

    let mut rustdoc_json_path = get_target_directory(manifest_path.as_ref());
    rustdoc_json_path.push("doc");
    rustdoc_json_path.push(format!("{}.json", package_name(manifest_path)));
    rustdoc_json_path
}

/// Figures out the name of the library crate in the current directory by
/// looking inside `Cargo.toml`
fn package_name(path: impl AsRef<Path>) -> String {
    let manifest = cargo_toml::Manifest::from_path(&path).expect("valid manifest");
    manifest
        .package
        .expect("[package] is declared in Cargo.toml")
        .name
}

/// Typically returns the absolute path to the regular cargo `./target` directory.
fn get_target_directory(manifest_path: &Path) -> PathBuf {
    let mut metadata_cmd = cargo_metadata::MetadataCommand::new();
    metadata_cmd.manifest_path(&manifest_path);
    let metadata = metadata_cmd.exec().unwrap();

    metadata.target_directory.as_std_path().to_owned()
}
