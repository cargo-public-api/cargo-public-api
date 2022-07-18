use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};

/// Synchronously do a `git checkout` of `commit`.
pub(crate) fn git_checkout(commit: &str, git_root: &Path) -> Result<()> {
    let mut command = std::process::Command::new("git");
    command.current_dir(git_root);
    command.args(["checkout", commit]);
    if command.spawn()?.wait()?.success() {
        Ok(())
    } else {
        Err(anyhow!(
            "Failed to git checkout {}, see error message on stdout/stderr.",
            commit,
        ))
    }
}

/// Goes up the chain of parents and looks for a `.git` dir.
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
