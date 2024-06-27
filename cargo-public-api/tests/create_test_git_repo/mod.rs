use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

/// The cargo-public-api project is meant to be used in git repositories. Since
/// it does "destructive" operations (checkout out arbitrary commits to the
/// working tree) we can't use the git repo that hosts this file. We need a
/// special git repo for testing purposes. That also allows tests to run
/// concurrently and independently.
///
/// This function creates a git repo for testing purposes from scratch, by
/// turning pre-made versions of an `example_api` into commits and tags.
///
/// `dest_dir` - The directory where a git repository shall be created.
///
/// `test_api_dir` - The `testsuite/test-apis` source dir. Used to find the path to the
/// `example_api`s.
pub fn create_test_git_repo(dest_dir: impl AsRef<Path>, dirs_and_tags: &[(&str, &str)]) {
    // Make sure the dest exists
    fs::create_dir_all(&dest_dir).unwrap();

    // Make the following git commands pretend we are in `dest_dir`
    let git = || {
        let mut cmd = Command::new("git");
        cmd.arg("-C");
        cmd.arg(dest_dir.as_ref());
        cmd
    };

    // First step: `git init`. Be quiet to avoid noisy test output.
    run(git().args(["init", "--quiet", "--initial-branch", "main"]));

    // Needed to prevent errors in CI
    run(git().args(["config", "user.email", "cargo-public-api@example.com"]));
    run(git().args(["config", "user.name", "Cargo Public"]));

    // Now go through all directories and create git commits and tags from them
    for dir_and_tag in dirs_and_tags {
        let dir_name = dir_and_tag.0;
        let tag_name = dir_and_tag.1;

        let copy_to_dest = |name| {
            let mut from = PathBuf::from("../testsuite/test-apis");
            from.push(dir_name);
            from.push(name);

            let mut to = PathBuf::from(dest_dir.as_ref());
            to.push(name);

            fs::copy(from, to).unwrap();
        };

        let mut src = PathBuf::from(dest_dir.as_ref());
        src.push("src");
        fs::create_dir_all(src).unwrap();

        copy_to_dest("Cargo.toml");
        copy_to_dest("src/lib.rs");

        run(git().args(["add", "."]));
        run(git()
            .args(["-c", "commit.gpgsign=false", "commit", "--quiet", "-m"])
            .arg(tag_name));
        run(git().arg("tag").arg(tag_name));
    }
}

fn run(cmd: &mut Command) {
    cmd.status().unwrap();
}
