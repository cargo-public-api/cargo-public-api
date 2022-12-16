use rustdoc_json::{Builder, PackageTarget};

#[test]
fn test_alternative_package_targets() {
    let targets = [
        PackageTarget::Bin("comprehensive_api"),
        PackageTarget::Bin("main2"),
        PackageTarget::Test("test"),
        PackageTarget::Example("example"),
        PackageTarget::Bench("bench"),
    ];

    for target in targets {
        let path = Builder::default()
            .toolchain("nightly".to_owned())
            .manifest_path("../test-apis/comprehensive_api/Cargo.toml")
            .quiet(true) // Make it less noisy to run tests
            .package_target(target)
            .build()
            .unwrap();

        assert!(path.exists());
    }
}
