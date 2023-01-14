/// Keep this code in sync with the code in `../../README.md`
#[test]
fn public_api() {
    // Install a proper nightly toolchain if it is missing
    rustup_toolchain::ensure_installed(public_api::MINIMUM_NIGHTLY_VERSION).unwrap();

    // Build rustdoc JSON
    let rustdoc_json = rustdoc_json::Builder::default()
        .toolchain(public_api::MINIMUM_NIGHTLY_VERSION)
        .build()
        .unwrap();

    // Derive the public API from the rustdoc JSON
    let public_api = public_api::Builder::from_rustdoc_json(rustdoc_json)
        .build()
        .unwrap();

    // Assert that the public API looks correct
    expect_test::expect_file!["public-api.txt"].assert_eq(&public_api.to_string());
}
