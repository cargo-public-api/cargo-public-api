pub fn repo_path(relative_path_from_repo_root: &str) -> std::path::PathBuf {
    let mut repo_root = std::env::current_dir().unwrap();

    // `find -name testsuite -type d` only gets one hit and it is in the root.
    while !repo_root.join("testsuite").is_dir() {
        let _ = repo_root.pop();
    }
    repo_root.join(relative_path_from_repo_root)
}

pub fn build_public_api(package_name: &str) -> public_api::PublicApi {
    // Install a compatible nightly toolchain if it is missing
    rustup_toolchain::install(public_api::MINIMUM_NIGHTLY_RUST_VERSION).unwrap();

    // Build rustdoc JSON with a separate target dir for increased parallelism
    // and reduced risk of cargo removing files we want to keep
    let target_dir = repo_path("target2/public_api").join(package_name);
    let manifest = repo_path(package_name).join("Cargo.toml");
    let rustdoc_json_path = rustdoc_json::Builder::default()
        .toolchain(public_api::MINIMUM_NIGHTLY_RUST_VERSION)
        .target_dir(&target_dir)
        .manifest_path(&manifest)
        .build()
        .unwrap_or_else(|e| panic!("{e} manifest={manifest:?} target_dir={target_dir:?}"));

    // Derive the public API from the rustdoc JSON
    public_api::Builder::from_rustdoc_json(&rustdoc_json_path)
        .build()
        .unwrap_or_else(|e| {
            panic!(
                "{e} manifest={manifest:?} target_dir={target_dir:?} rustdoc_json={rustdoc_json_path:?}"
            )
        })
}
