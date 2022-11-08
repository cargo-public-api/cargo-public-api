#[test]
fn ensure_workspace_inheritance_works() {
    let path = rustdoc_json::Builder::default()
        .toolchain("nightly".to_owned())
        .manifest_path("../cargo-public-api/tests/virtual-manifest/workspace-version/Cargo.toml")
        .build()
        .unwrap();

    assert_eq!(
        path,
        std::env::current_dir()
            .unwrap()
            .parent()
            .unwrap()
            .join("cargo-public-api/tests/virtual-manifest/target/doc/workspace_version.json")
    );
}
