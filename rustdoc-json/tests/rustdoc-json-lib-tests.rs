use public_api::{Options, PublicApi};
use rustdoc_json::{Builder, PackageTarget};

#[test]
fn public_api() -> Result<(), Box<dyn std::error::Error>> {
    let rustdoc_json = Builder::default().toolchain("nightly".to_owned()).build()?;

    let public_api = PublicApi::from_rustdoc_json(rustdoc_json, Options::default())?;

    expect_test::expect_file!["../public-api.txt"].assert_eq(&public_api.to_string());

    Ok(())
}

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
            .manifest_path("tests/test_crate/Cargo.toml")
            .quiet(true) // Make it less noisy to run tests
            .package_target(target)
            .build()
            .unwrap();

        assert!(path.exists());
    }
}
