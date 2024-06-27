#[test]
fn public_api() {
    expect_test::expect_file!["public-api.txt"]
        .assert_eq(&crate::utils::build_public_api("rustup-toolchain").to_string());
}
