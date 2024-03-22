/// Returns true if it seems like the currently active toolchain is the stable
/// toolchain.
///
/// See <https://rust-lang.github.io/rustup/overrides.html> for some
/// more info of how different toolchains can be activated.
pub fn is_probably_stable() -> bool {
    let mut cmd = std::process::Command::new("cargo");
    cmd.arg("--version");

    let Ok(output) = cmd.output() else {
        return false;
    };

    let Ok(version) = String::from_utf8(output.stdout) else {
        return false;
    };

    version.starts_with("cargo 1") && !version.contains("nightly")
}

/// Returns the current toolchain if it is overridden by the environment
/// variable `RUSTUP_TOOLCHAIN` which `rustup` sets when its proxies are invoked
/// with the `+toolchain` arg, e.g. `cargo +nightly ...`.
pub fn from_rustup() -> Option<String> {
    let mut cmd = std::process::Command::new("rustup");
    cmd.args(["show", "active-toolchain"]);
    cmd.env_remove("RUSTUP_TOOLCHAIN");

    let output = String::from_utf8(cmd.output().ok()?.stdout).ok()?;

    output
        .split(char::is_whitespace)
        .next()
        .and_then(|default| {
            let toolchain = std::env::var("RUSTUP_TOOLCHAIN").ok();
            if toolchain.as_deref() == Some(default) {
                None
            } else {
                toolchain
            }
        })
}
