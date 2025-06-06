//! Utilities for working with `rustup` toolchains.
//!
//! # Ensuring a toolchain is installed
//!
//! This checks if a toolchain is installed, and installs it if not.
//!
//! ```no_run
//! rustup_toolchain::ensure_installed("nightly").unwrap();
//! ```

#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
/// Enumerates all errors that can currently occur within this crate.
pub enum Error {
    /// Some kind of IO error occurred.
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    /// The lock used to work around <https://github.com/rust-lang/rustup/issues/988> has been poisoned
    #[error(
        "The lock used to work around https://github.com/rust-lang/rustup/issues/988 has been poisoned"
    )]
    StdSyncPoisonError,

    /// `rustup toolchain install ...` failed for some reason
    #[error("`rustup toolchain install ...` failed for some reason")]
    RustupToolchainInstallError,
}

/// Shorthand for [`std::result::Result<T, rustup_toolchain::Error>`].
pub type Result<T> = std::result::Result<T, Error>;

/// As a workaround for [Rustup (including proxies) is not safe for concurrent
/// use](https://github.com/rust-lang/rustup/issues/988) we keep a per-process
/// global lock.
static RUSTUP_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Installs a toolchain if it is not already installed.
///
/// As a workaround for [Rustup (including proxies) is not safe for concurrent
/// use](https://github.com/rust-lang/rustup/issues/988) this function is
/// protected by a process-global lock. If you use multiple processes, you need
/// to prevent concurrent `rustup` usage yourself.
///
/// # Errors
///
/// If `rustup` is not installed on your system, for example.
pub fn install(toolchain: impl AsRef<str>) -> Result<()> {
    // The reason we check if the toolchain is installed rather than always
    // doing `rustup install toolchain` is because otherwise there will be noisy
    // "already installed" output from `rustup install toolchain`.
    if !is_installed(toolchain.as_ref())? {
        run_rustup_install(toolchain)?;
    }

    Ok(())
}

/// Deprecated
#[deprecated(since = "0.1.4", note = "Renamed to `install()` for brevity.")]
pub fn ensure_installed(toolchain: &str) -> Result<()> {
    install(toolchain)
}

/// Check if a toolchain is installed.
///
/// As a workaround [Rustup (including proxies) is not safe for concurrent
/// use](https://github.com/rust-lang/rustup/issues/988) this function is
/// protected by a process-global lock. If you use multiple processes, you need
/// to prevent concurrent `rustup` usage yourself.
///
/// # Errors
///
/// If `rustup` is not installed on your system, for example.
pub fn is_installed(toolchain: &str) -> Result<bool> {
    let _guard = RUSTUP_MUTEX.lock().map_err(|_| Error::StdSyncPoisonError)?;

    Ok(std::process::Command::new("rustup")
        .arg("run")
        .arg(toolchain)
        .arg("cargo")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()?
        .success())
}

fn run_rustup_install(toolchain: impl AsRef<str>) -> Result<()> {
    let _guard = RUSTUP_MUTEX.lock().map_err(|_| Error::StdSyncPoisonError)?;

    let status = std::process::Command::new("rustup")
        .arg("toolchain")
        .arg("install")
        .arg("--no-self-update")
        .arg("--profile")
        .arg("minimal")
        .arg(toolchain.as_ref())
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(Error::RustupToolchainInstallError)
    }
}
