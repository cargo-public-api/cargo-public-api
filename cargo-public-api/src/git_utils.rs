use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};

/// Synchronously do a `git checkout` of `commit`.
/// Returns the name of the original branch.
pub(crate) fn git_checkout(commit: &str, git_root: &Path, quiet: bool) -> Result<String> {
    let original_branch = current_branch(&git_root)?;

    let mut command = std::process::Command::new("git");
    command.current_dir(git_root);
    command.args(["checkout", commit]);
    if quiet {
        command.arg("--quiet");
    }
    if command.spawn()?.wait()?.success() {
        Ok(original_branch)
    } else {
        Err(anyhow!(
            "Failed to git checkout {}, see error message on stdout/stderr.",
            commit,
        ))
    }
}

/// Goes up the chain of parents and looks for a `.git` dir.
#[allow(unused)] // It IS used!
pub(crate) fn git_root_from_manifest_path(manifest_path: &Path) -> Result<PathBuf> {
    let err_fn = || anyhow!("No `.git` dir when starting from `{:?}`.", &manifest_path);
    let start = std::fs::canonicalize(manifest_path).with_context(err_fn)?;
    let mut candidate_opt = start.parent();
    while let Some(candidate) = candidate_opt {
        if [candidate, Path::new(".git")]
            .iter()
            .collect::<PathBuf>()
            .exists()
        {
            return Ok(candidate.to_owned());
        }
        candidate_opt = candidate.parent();
    }
    Err(err_fn())
}

/// Returns the name of the current git branch.
pub(crate) fn current_branch(path: impl AsRef<Path>) -> Result<String> {
    let mut git = std::process::Command::new("git");
    git.current_dir(path);
    git.args(&["rev-parse", "--abbrev-ref", "HEAD"]);

    let output = git.output()?;
    if output.status.success() && output.stderr.is_empty() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(anyhow!("Failed to get current branch: {:?}", output))
    }
}
