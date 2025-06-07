#![allow(unused)] // Used from many crates, but not all crates use all functions.

use std::{
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result, anyhow};

/// Synchronously do a `git checkout` of `commit`.
pub fn git_checkout(git_root: &Path, commit: &str, quiet: bool, force: bool) -> Result<()> {
    let mut command = Command::new("git");
    command.current_dir(git_root);
    command.args(["checkout", commit]);
    if quiet {
        command.arg("--quiet");
    }
    if force {
        command.arg("--force");
    }
    if command.spawn()?.wait()?.success() {
        Ok(())
    } else {
        Err(anyhow!(
            "Failed to `git checkout {}`, see error message on stdout/stderr.",
            commit,
        ))
    }
}

/// Goes up the chain of parents and looks for a `.git` dir.
pub fn git_root_from_manifest_path(manifest_path: &Path) -> Result<PathBuf> {
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

pub fn current_branch_or_commit(path: impl AsRef<Path>) -> Result<String> {
    let current_branch = current_branch(&path)?;
    let current_commit = current_commit(&path)?;
    Ok(current_branch.unwrap_or(current_commit))
}

/// Returns the name of the current git branch. Or `None` if there is no current
/// branch.
pub fn current_branch(path: impl AsRef<Path>) -> Result<Option<String>> {
    let branch = trimmed_git_stdout(path, &["rev-parse", "--abbrev-ref", "HEAD"])?;
    if &branch == "HEAD" {
        Ok(None)
    } else {
        Ok(Some(branch))
    }
}
/// Returns the current commit hash.
pub fn current_commit(path: impl AsRef<Path>) -> Result<String> {
    trimmed_git_stdout(path, &["rev-parse", "--short", "HEAD"])
}

fn trimmed_git_stdout(path: impl AsRef<Path>, args: &[&str]) -> Result<String> {
    let mut git = Command::new("git");
    git.current_dir(path);
    git.args(args);
    trimmed_stdout(git)
}

fn trimmed_stdout(mut cmd: Command) -> Result<String> {
    let output = cmd.output()?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(anyhow!(String::from_utf8_lossy(&output.stderr).to_string()))
    }
}

/// Resolves a git reference provided at the CLI to an actual commit, allowing
/// us to validate refs and use "relative" values like HEAD and more.
pub fn resolve_ref(path: impl AsRef<Path>, committish: &str) -> Result<String> {
    trimmed_git_stdout(path, &["rev-parse", committish])
}
