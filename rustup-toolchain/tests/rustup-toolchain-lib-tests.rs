use public_api::{Options, PublicApi};

#[test]
fn public_api() -> Result<(), Box<dyn std::error::Error>> {
    let rustdoc_json = rustdoc_json::Builder::default()
        .toolchain("nightly".to_owned())
        .build()?;

    let public_api = PublicApi::from_rustdoc_json(rustdoc_json, Options::default())?;

    expect_test::expect_file!["../public-api.txt"].assert_eq(&public_api.to_string());

    Ok(())
}
