/// Test that there is no error when building rustdoc JSON for a package that
/// uses workspace inheritance
#[test]
fn ensure_workspace_inheritance_works() {
    let path = rustdoc_json::Builder::default()
        .toolchain("nightly".to_owned())
        .manifest_path("../test-apis/workspace-inheritance/package-with-inheritance/Cargo.toml")
        .build()
        .unwrap();

    assert_eq!(
        path,
        std::env::current_dir()
            .unwrap()
            .parent()
            .unwrap()
            .join("test-apis/workspace-inheritance/target/doc/package_with_inheritance.json")
    );
}
