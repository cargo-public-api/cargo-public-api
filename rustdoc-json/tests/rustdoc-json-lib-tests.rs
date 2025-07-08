use rustdoc_json::PackageTarget;

/// Test that there is no error when building rustdoc JSON for a package that
/// uses workspace inheritance
#[test]
fn ensure_workspace_inheritance_works() {
    let path = rustdoc_json::Builder::default()
        .toolchain("nightly")
        .manifest_path("../test-apis/workspace-inheritance/package-with-inheritance/Cargo.toml")
        .quiet(true) // Make it less noisy to run tests
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

#[test]
fn resolver_3() {
    rustdoc_json::Builder::default()
        .toolchain("nightly")
        .manifest_path("../test-apis/resolver-3/Cargo.toml")
        .quiet(true) // Make it less noisy to run tests
        .build()
        .unwrap();
}

#[test]
fn package_target_bin() {
    test_alternative_package_target(PackageTarget::Bin("test_crate".into()));
}

#[test]
fn package_target_bin_with_hyphen() {
    test_alternative_package_target(PackageTarget::Bin("with-hyphen".into()));
}

#[test]
fn package_target_bin_2() {
    test_alternative_package_target(PackageTarget::Bin("main2".into()));
}

#[test]
fn package_target_test() {
    test_alternative_package_target(PackageTarget::Test("test".into()));
}

#[test]
fn package_target_example() {
    test_alternative_package_target(PackageTarget::Example("example".into()));
}

#[test]
fn package_target_bench() {
    test_alternative_package_target(PackageTarget::Bench("bench".into()));
}

fn test_alternative_package_target(package_target: PackageTarget) {
    let target_dir = tempfile::tempdir().unwrap();

    let path = rustdoc_json::Builder::default()
        .toolchain("nightly")
        .manifest_path("tests/test_crates/test_crate/Cargo.toml")
        .quiet(true) // Make it less noisy to run tests
        .package_target(package_target)
        .target_dir(&target_dir)
        .build()
        .unwrap();

    assert!(path.exists());
}

/// The cargo test framework can't capture stderr from child processes. So use a
/// simple program and capture its stderr to test if `silent(true)` works.
#[test]
fn silent_build() {
    use assert_cmd::Command;
    use predicates::str::contains;

    let stderr_substring_if_not_silent = "invalid/because/we/want/it/to/fail/Cargo.toml";
    Command::cargo_bin("test-silent-build")
        .unwrap()
        .assert()
        .stderr(contains(stderr_substring_if_not_silent))
        .failure();

    Command::cargo_bin("test-silent-build")
        .unwrap()
        .arg("--silent")
        .assert()
        .try_stderr(contains(stderr_substring_if_not_silent))
        .expect_err(&format!(
            "Found `{stderr_substring_if_not_silent}` in stderr, but stderr should be silent!"
        ));
}

#[test]
fn capture_output() {
    let target_dir = tempfile::tempdir().unwrap();
    let mut stdout = vec![];
    let mut stderr = vec![];

    let result = rustdoc_json::Builder::default()
        .toolchain("nightly")
        .manifest_path("tests/test_crates/test_crate_error/Cargo.toml")
        .quiet(true) // Make it less noisy to run tests
        .color(rustdoc_json::Color::Never)
        .target_dir(&target_dir)
        .build_with_captured_output(&mut stdout, &mut stderr);

    assert!(matches!(
        result,
        Err(rustdoc_json::BuildError::BuildRustdocJsonError)
    ));
    assert!(stdout.is_empty());

    let stderr = String::from_utf8(stderr).unwrap();

    assert!(
        stderr.contains("error: this file contains an unclosed delimiter"),
        "Got stderr: {stderr}",
    );
}
