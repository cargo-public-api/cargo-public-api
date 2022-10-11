// deny in CI, only warn here
#![warn(clippy::all, clippy::pedantic)]

use std::io::stdout;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use arg_types::{Color, DenyMethod};
use plain::Plain;
use public_api::diff::PublicItemsDiff;
use public_api::{Options, PublicApi, PublicItem, MINIMUM_RUSTDOC_JSON_VERSION};

use clap::Parser;
use rustdoc_json::BuildError;

mod arg_types;
mod error;
mod git_utils;
mod plain;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
#[allow(clippy::struct_excessive_bools)]
pub struct Args {
    /// Path to `Cargo.toml`.
    #[clap(long, name = "PATH", default_value = "Cargo.toml", parse(from_os_str))]
    manifest_path: PathBuf,

    /// Raise this flag to make items part of blanket implementations such as
    /// `impl<T> Any for T`, `impl<T> Borrow<T> for T`, and `impl<T, U> Into<U>
    /// for T where U: From<T>` be included in the list of public items of a
    /// crate.
    ///
    /// Blanket implementations are not included by default since the vast
    /// majority of users will find the presence of these items to just
    /// constitute noise, even if they formally are part of the public API of a
    /// crate.
    #[clap(long)]
    with_blanket_implementations: bool,

    /// Usage: --diff-git-checkouts <COMMIT_1> <COMMIT_2>
    ///
    /// Allows to diff the public API across two different commits. The
    /// following steps are performed:
    ///
    /// 1. Remember the current branch/commit
    ///
    /// 2. Do a literal in-tree, in-place `git checkout` of the first commit
    ///
    /// 3. Collect public API
    ///
    /// 4. Do a literal in-tree, in-place `git checkout` of the second commit
    ///
    /// 5. Collect public API
    ///
    /// 6. Print the diff between public API in step 2 and step 4
    ///
    /// 7. Restore the original branch/commit
    ///
    /// If you have local changes, git will refuse to do `git checkout`, so your
    /// work will not be discarded.
    ///
    /// Do not use non-fixed commit references such as `HEAD^` since the meaning
    /// of `HEAD^` is different depending on what commit is the current commit.
    ///
    /// Using the current git repo has the benefit of making it likely for the
    /// build to succeed. If we e.g. were to git clone a temporary copy of a
    /// commit ourselves, the risk is high that additional steps are needed
    /// before a build can succeed. Such as the need to set up git submodules.
    #[clap(long, min_values = 2, max_values = 2)]
    diff_git_checkouts: Option<Vec<String>>,

    /// Usage: --diff-rustdoc-json <RUSTDOC_JSON_PATH_1> <RUSTDOC_JSON_PATH_2>
    ///
    /// Diff the public API across two different rustdoc JSON files.
    #[clap(long, min_values = 2, max_values = 2, hide = true)]
    diff_rustdoc_json: Option<Vec<String>>,

    /// Exit with failure if the specified API diff is detected.
    ///
    /// * all = deny added, changed, and removed public items in the API
    ///
    /// * added = deny added public items to the API
    ///
    /// * changed = deny changed public items in the API
    ///
    /// * removed = deny removed public items from the API
    ///
    /// They can also be combined. For example, to only allow additions to the
    /// API, use `--deny=added --deny=changed`.
    #[clap(long, arg_enum)]
    deny: Option<Vec<DenyMethod>>,

    /// Whether or not to use colors. You can select between "auto", "never", "always".
    /// If "auto" (the default), colors will be used if stdout is a terminal. If you pipe
    /// the output to a file, colors will be disabled by default.
    #[clap(long, default_value = "auto")]
    color: Color,

    /// Show detailed info about processing. For debugging purposes. The output
    /// is not stable and can change across patch versions.
    #[clap(long, hide = true)]
    verbose: bool,

    /// Allows you to build rustdoc JSON with a toolchain other than `nightly`.
    ///
    /// Consider using `cargo +toolchain public-api` instead.
    ///
    /// Useful if you have built a toolchain from source for example, or if you
    /// want to use a fixed toolchain in CI.
    #[clap(long, validator = |s: &str| if !s.starts_with('+') { Ok(()) } else { Err("toolchain must not start with a `+`")} )]
    toolchain: Option<String>,

    /// Build for the target triple
    #[clap(long)]
    target: Option<String>,

    /// Space or comma separated list of features to activate
    #[clap(long, short = 'F', min_values = 1)]
    features: Vec<String>,

    #[clap(long)]
    /// Activate all available features
    all_features: bool,

    #[clap(long)]
    /// Do not activate the `default` feature
    no_default_features: bool,

    /// Package to document
    #[clap(long, short)]
    package: Option<String>,

    /// Forwarded to rustdoc JSON build command
    #[clap(long, hide = true)]
    cap_lints: Option<String>,
}

/// After listing or diffing, we might want to do some extra work. This struct
/// keeps track of what to do.
struct PostProcessing {
    /// The `--deny` arg allows the user to disallow the occurrence of API
    /// changes. If this field is set, we are to check that the diff is allowed.
    diff_to_check: Option<PublicItemsDiff>,

    /// Doing a `--diff-git-checkouts` involves doing `git checkout`s.
    /// Afterwards, we want to restore the original branch the user was on, to
    /// not mess up their work tree.
    branch_to_restore: Option<String>,
}

fn main_() -> Result<()> {
    let mut args = get_args();

    // check if using a stable compiler, and use nightly if it is.
    if active_toolchain_is_probably_stable(args.toolchain.as_deref()) {
        if let Some(toolchain) = args
            .toolchain
            .or_else(active_toolchain_is_environment_override)
        {
            eprintln!("Warning: using the `{toolchain}` toolchain for gathering the public api is not possible, switching to `nightly`");
        }
        args.toolchain = Some("nightly".to_owned());
    }

    let post_processing = if let Some(commits) = &args.diff_git_checkouts {
        print_diff_between_two_commits(&args, commits)?
    } else if let Some(files) = &args.diff_rustdoc_json {
        print_diff_between_two_rustdoc_json_files(&args, files)?
    } else {
        print_public_items_of_current_commit(&args)?
    };

    post_processing.perform(&args)
}

/// Returns true if it seems like the currently active toolchain is the stable
/// toolchain.
///
/// See <https://rust-lang.github.io/rustup/overrides.html> for some
/// more info of how different toolchains can be activated.
fn active_toolchain_is_probably_stable(toolchain: Option<&str>) -> bool {
    let mut cmd = toolchain.map_or_else(
        || std::process::Command::new("cargo"),
        |toolchain| {
            let mut cmd = std::process::Command::new("rustup");
            cmd.args(["run", toolchain, "cargo"]);
            cmd
        },
    );
    cmd.arg("--version");

    let output = match cmd.output() {
        Ok(output) => output,
        Err(_) => return false,
    };

    let version = match String::from_utf8(output.stdout) {
        Ok(version) => version,
        Err(_) => return false,
    };

    version.starts_with("cargo 1") && !version.contains("nightly")
}

/// returns the current toolchain if it was overriden by environment
fn active_toolchain_is_environment_override() -> Option<String> {
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

fn check_diff(args: &Args, diff: &Option<PublicItemsDiff>) -> Result<()> {
    match (&args.deny, diff) {
        // We were requested to deny diffs, so make sure there is no diff
        (Some(deny), Some(diff)) => {
            let mut violations = crate::error::Violations::new();
            for d in deny {
                if d.deny_added() && !diff.added.is_empty() {
                    violations.extend_added(diff.added.iter().cloned());
                }
                if d.deny_changed() && !diff.changed.is_empty() {
                    violations.extend_changed(diff.changed.iter().cloned());
                }
                if d.deny_removed() && !diff.removed.is_empty() {
                    violations.extend_removed(diff.removed.iter().cloned());
                }
            }

            if violations.is_empty() {
                Ok(())
            } else {
                Err(anyhow!(error::Error::DiffDenied(violations)))
            }
        }

        // We were requested to deny diffs, but we did not calculate a diff!
        (Some(_), None) => Err(anyhow!("`--deny` can only be used when diffing")),

        // No diff related stuff to care about, all is Ok
        _ => Ok(()),
    }
}

fn print_public_items_of_current_commit(args: &Args) -> Result<PostProcessing> {
    let (public_api, branch_to_restore) = collect_public_api_from_commit(args, None)?;

    if args.verbose {
        public_api.missing_item_ids.iter().for_each(|i| {
            println!("NOTE: rustdoc JSON missing referenced item with ID \"{i}\"");
        });
    }

    Plain::print_items(&mut stdout(), args, public_api.items)?;

    Ok(PostProcessing {
        diff_to_check: None,
        branch_to_restore,
    })
}

fn print_diff_between_two_commits(args: &Args, commits: &[String]) -> Result<PostProcessing> {
    let old_commit = commits.get(0).expect("clap makes sure first commit exist");
    let (old, branch_to_restore) = collect_public_api_from_commit(args, Some(old_commit))?;

    let new_commit = commits.get(1).expect("clap makes sure second commit exist");
    let (new, _) = collect_public_api_from_commit(args, Some(new_commit))?;

    let diff_to_check = Some(print_diff(args, old.items, new.items)?);

    Ok(PostProcessing {
        diff_to_check,
        branch_to_restore,
    })
}

fn print_diff_between_two_rustdoc_json_files(
    args: &Args,
    files: &[String],
) -> Result<PostProcessing> {
    let options = get_options(args);

    let old_file = files.get(0).expect("clap makes sure first file exists");
    let old = public_api_from_rustdoc_json_path(old_file, options)?;

    let new_file = files.get(1).expect("clap makes sure second file exists");
    let new = public_api_from_rustdoc_json_path(new_file, options)?;

    let diff_to_check = Some(print_diff(args, old.items, new.items)?);

    Ok(PostProcessing {
        diff_to_check,
        branch_to_restore: None,
    })
}

fn print_diff(args: &Args, old: Vec<PublicItem>, new: Vec<PublicItem>) -> Result<PublicItemsDiff> {
    let diff = PublicItemsDiff::between(old, new);
    Plain::print_diff(&mut stdout(), args, &diff)?;

    Ok(diff)
}

impl PostProcessing {
    fn perform(&self, args: &Args) -> Result<()> {
        if let Some(branch_to_restore) = &self.branch_to_restore {
            git_utils::git_checkout(branch_to_restore, &args.git_root()?, !args.verbose)?;
        }

        check_diff(args, &self.diff_to_check)
    }
}

impl Args {
    fn git_root(&self) -> Result<PathBuf> {
        git_utils::git_root_from_manifest_path(self.manifest_path.as_path())
    }
}

/// Get CLI args via `clap` while also handling when we are invoked as a cargo
/// subcommand. When the user runs `cargo public-api -a -b -c` our args will be
/// `cargo-public-api public-api -a -b -c`.
fn get_args() -> Args {
    let args = std::env::args_os()
        .enumerate()
        .filter(|(index, arg)| *index != 1 || arg != "public-api")
        .map(|(_, arg)| arg);

    Args::parse_from(args)
}

/// Figure out what [`Options`] to pass to
/// [`public_api::PublicApi::from_rustdoc_json_str`] based on our
/// [`Args`]
fn get_options(args: &Args) -> Options {
    let mut options = Options::default();
    options.with_blanket_implementations = args.with_blanket_implementations;
    options
}

/// Collects public items from either the current commit or a given commit. If
/// `commit` is `Some` and thus a `git checkout` will be made, also return the
/// original branch.
fn collect_public_api_from_commit(
    args: &Args,
    commit: Option<&str>,
) -> Result<(PublicApi, Option<String>)> {
    // Do a git checkout of a specific commit unless we are supposed to simply
    // use the current commit
    let original_branch = if let Some(commit) = commit {
        Some(git_utils::git_checkout(
            commit,
            &args.git_root()?,
            !args.verbose,
        )?)
    } else {
        None
    };
    let mut builder = rustdoc_json::Builder::default()
        .toolchain(args.toolchain.clone())
        .manifest_path(&args.manifest_path)
        .all_features(args.all_features)
        .no_default_features(args.no_default_features)
        .features(&args.features);
    if let Some(target) = &args.target {
        builder = builder.target(target.clone());
    }

    if let Some(package) = &args.package {
        builder = builder.package(package);
    }

    if let Some(cap_lints) = &args.cap_lints {
        builder = builder.cap_lints(Some(cap_lints));
    }

    let json_path = match builder.build() {
        Err(BuildError::VirtualManifest(manifest_path)) => virtual_manifest_error(&manifest_path)?,
        res => res?,
    };
    if args.verbose {
        println!("Processing {:?}", json_path);
    }
    let options = get_options(args);

    Ok((
        public_api_from_rustdoc_json_path(json_path, options)?,
        original_branch,
    ))
}

fn public_api_from_rustdoc_json_path<T: AsRef<Path>>(
    json_path: T,
    options: Options,
) -> Result<PublicApi> {
    let rustdoc_json = &std::fs::read_to_string(&json_path)
        .with_context(|| format!("Failed to read rustdoc JSON at {:?}", json_path.as_ref()))?;

    PublicApi::from_rustdoc_json_str(rustdoc_json, options).with_context(|| {
        format!(
            "Failed to parse rustdoc JSON at {:?}.\n\
            This version of `cargo public-api` requires at least:\n\n    {}\n\n\
            If you have that, it might be `cargo public-api` that is out of date. Try\n\
            to install the latest version with `cargo install cargo-public-api`. If the\n\
            issue remains, please report at\n\n    https://github.com/Enselic/cargo-public-api/issues",
            json_path.as_ref(),
            MINIMUM_RUSTDOC_JSON_VERSION,
        )
    })
}

fn virtual_manifest_error(manifest_path: &Path) -> Result<PathBuf> {
    Err(anyhow!(
        "`{:?}` is a virtual manifest.

Listing or diffing the public API of an entire workspace is not supported.

Try

    cargo public-api -p specific-crate
",
        manifest_path
    ))
}

/// Wrapper to handle <https://github.com/rust-lang/rust/issues/46016>
fn main() -> Result<()> {
    match main_() {
        Err(e) => match e.root_cause().downcast_ref::<std::io::Error>() {
            Some(io_error) if io_error.kind() == std::io::ErrorKind::BrokenPipe => {
                std::process::exit(141)
            }
            _ => Err(e),
        },
        result => result,
    }
}
