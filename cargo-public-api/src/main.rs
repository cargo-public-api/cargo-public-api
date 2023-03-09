// deny in CI, only warn here
#![warn(clippy::all, clippy::pedantic)]

use std::ffi::OsString;
use std::io::stdout;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Result};
use api_source::{ApiSource, Commit, CurrentDir, PublishedCrate, RustdocJson};
use arg_types::{Color, DenyMethod, Omit};
use git_utils::current_branch_or_commit;
use plain::Plain;
use public_api::diff::PublicApiDiff;

use clap::{CommandFactory, Parser};

mod api_source;
mod arg_types;
mod error;
mod git_utils;
mod plain;
mod published_crate;
mod toolchain;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None, bin_name = "cargo public-api")]
#[allow(clippy::struct_excessive_bools)]
pub struct Args {
    /// Path to `Cargo.toml`.
    #[arg(long, value_name = "PATH", default_value = "Cargo.toml")]
    manifest_path: PathBuf,

    /// Name of package in workspace to list or diff the public API for.
    #[arg(long, short)]
    package: Option<String>,

    /// Omit items that belong to Blanket Implementations and Auto Trait
    /// Implementations (first use), and Auto Derived Implementations (second
    /// use).
    ///
    /// This makes the output significantly less noisy and repetitive, at the
    /// cost of not fully describing the public API.
    ///
    /// Use `--omit ...` for more control.
    #[arg(short, long, action = clap::ArgAction::Count)]
    simplified: u8,

    /// Omit certain kinds of items from the output to make it less noisy
    #[arg(long, value_enum)]
    omit: Option<Vec<Omit>>,

    /// Space or comma separated list of features to activate
    #[arg(long, short = 'F', num_args = 1..)]
    features: Vec<String>,

    /// Activate all available features
    #[arg(long)]
    all_features: bool,

    /// Do not activate the `default` feature
    #[arg(long)]
    no_default_features: bool,

    /// Build for the target triple
    #[arg(long)]
    target: Option<String>,

    /// How to color the output. By default, `--color=auto` is active. Using
    /// just `--color` without an arg is equivalent to `--color=always`.
    #[allow(clippy::option_option)]
    #[arg(long, value_enum)]
    color: Option<Option<Color>>,

    /// List the public API based on the given rustdoc JSON file.
    ///
    /// Example:
    ///
    /// First do
    ///
    ///     rustup component add rust-docs-json --toolchain nightly
    ///
    /// and then
    ///
    ///     cargo public-api --rustdoc-json ~/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/share/doc/rust/json/std.json
    ///
    #[arg(long, value_name = "RUSTDOC_JSON_PATH", hide = true)]
    rustdoc_json: Option<String>,

    /// Show detailed info about processing.
    ///
    /// For debugging purposes. The output is not stable and can change across
    /// patch versions.
    #[arg(long, hide = true)]
    verbose: bool,

    /// Include the so called "sorting prefix" that makes items grouped in a
    /// nice way.
    ///
    /// Only intended for debugging this tool.
    #[arg(long, hide = true)]
    debug_sorting: bool,

    /// Put rustdoc JSON build artifacts in the specified dir instead of in
    /// `./target`. Option hidden by default because it will typically not be
    /// needed by users. Mainly useful to allow tests to run in parallel.
    #[arg(long, value_name = "PATH", hide = true)]
    target_dir: Option<PathBuf>,

    /// Build rustdoc JSON with a toolchain other than `nightly`.
    ///
    /// Consider using `cargo +toolchain public-api` instead.
    ///
    /// Useful if you have built a toolchain from source for example, or if you
    /// want to use a fixed toolchain in CI.
    #[arg(long, value_parser = parse_toolchain, hide = true)]
    toolchain: Option<String>,

    /// Forwarded to rustdoc JSON build command
    #[arg(long, hide = true)]
    cap_lints: Option<String>,

    #[command(subcommand)]
    subcommand: Option<Subcommand>,
}

/// The subcommand used for diffing.
#[derive(Parser, Debug)]
#[allow(clippy::struct_excessive_bools)]
struct DiffArgs {
    /// Exit with failure if the specified API diff is detected.
    ///
    /// Can be combined. For example, to only allow additions to the API, use
    /// `--deny=added --deny=changed`.
    #[arg(long, value_enum)]
    deny: Option<Vec<DenyMethod>>,

    /// Force the diff. For example, when diffing commits, enabling this option
    /// will discard working tree changes during git checkouts of other commits.
    #[arg(long)]
    force: bool,

    /// What to diff. See `cargo public-api diff --help` for examples and more
    /// info.
    args: Vec<String>,
}

#[derive(clap::Subcommand, Debug)]
enum Subcommand {
    /// Diff the public API against a published version of the crate, or between commits.
    ///
    /// If the diffing
    ///
    /// * arg looks like a specific version string (`x.y.z`) then the diff will be between that published
    ///   version of the crate and the working directory.
    ///
    /// * arg has `..` like in `tag1..tag2` then the public API of each individual git commit will be
    ///   diffed. See below for how that works.
    ///
    /// * args end with `.json` like in `file1.json file2.json` then rustdoc JSON file diffing will be
    ///   performed.
    ///
    ///
    /// EXAMPLES:
    /// =========
    ///
    /// Diffing a published version of a crate against the current working tree:
    ///
    ///     cargo public-api diff 1.2.3
    ///
    /// Diffing the latest version of a crate against the current working tree:
    ///
    ///     cargo public-api diff latest
    ///
    /// Diffing working tree against a published version of a specific crate in the workspace:
    ///
    ///     cargo public-api -p specific-crate diff 1.2.3
    ///
    /// Diffing between commits:
    ///
    ///     cargo public-api diff v0.2.0..v0.3.0
    ///
    /// Diffing between rustdoc JSON files:
    ///
    ///     cargo public-api diff first.json second.json
    ///
    ///
    /// HOW COMMIT DIFFING WORKS:
    /// =========================
    ///
    /// When diffing commits, the following steps are performed:
    ///
    /// 1. Remember the current branch/commit
    /// 2. Do a literal in-tree, in-place `git checkout` of the first commit
    /// 3. Collect public API
    /// 4. Do a literal in-tree, in-place `git checkout` of the second commit
    /// 5. Collect public API
    /// 6. Print the diff between public API in step 2 and step 4
    /// 7. Restore the original branch/commit
    ///
    /// If you have local changes, git will refuse to do `git checkout`, so your work will not be discarded.
    /// To force `git checkout`, use `--force`.
    ///
    /// Using the current git repo has the benefit of making it likely for the build to succeed. If we e.g.
    /// were to git clone a temporary copy of a commit ourselves, the risk is high that additional steps are
    /// needed before a build can succeed. Such as the need to set up git submodules.
    #[clap(verbatim_doc_comment)]
    Diff(DiffArgs),

    /// Generate completion scripts for many different shells.
    ///
    /// Example on how to generate and install the completion script for zsh:
    ///
    ///    $ mkdir ~/.zfunc
    ///    $ rustup completions zsh cargo > ~/.zfunc/_cargo
    ///    $ cargo public-api completions zsh > ~/.zfunc/_cargo-public-api
    ///    $ fpath+=~/.zfunc
    ///    $ autoload -U compinit && compinit
    ///    $ cargo public-api --{{Tab}}
    ///    --all-features         -- Activate all available features
    ///    --cap-lints            -- Forwarded to rustdoc JSON build command
    ///    --color                -- How to color the output. By default, `--color=auto` is active. Using just `--color` withou
    ///    --debug-sorting        -- Include the so called "sorting prefix" that makes items grouped in a nice way
    ///    [...]
    #[clap(verbatim_doc_comment)]
    Completions {
        #[arg(value_enum)]
        shell: clap_complete_command::Shell,
    },
}

enum MainTask {
    /// Print the public API of a crate.
    PrintList {
        api: Box<dyn ApiSource>,
    },
    /// Diff the public API of a crate.
    PrintDiff {
        old_api: Box<dyn ApiSource>,
        new_api: Box<dyn ApiSource>,
    },
    GenerateShellCompletionScript(clap_complete_command::Shell),
}

/// This represents an action that we want to do at some point.
pub enum Action {
    /// The `--deny` arg allows the user to disallow the occurrence of API
    /// changes. We are to check that the diff is allowed.
    CheckDiff {
        diff: PublicApiDiff,
        deny: Vec<DenyMethod>,
    },

    /// Doing a `--diff-git-checkouts` involves doing `git checkout`s.
    /// Afterwards, we want to restore the original branch the user was on, to
    /// not mess up their work tree.
    RestoreBranch { name: String },
}

/// The string used by users to request a diff of the latest (in semver terms)
/// published version of a given crate.
const LATEST_VERSION_ARG: &str = "latest";

fn main_() -> Result<()> {
    let args = get_args();

    // A list of actions to perform after we have listed or diffed. Typical
    // examples: restore a git branch or check that a diff is allowed
    let mut final_actions = vec![];

    // Now figure out our main task. Typically listing or diffing the public API
    let main_task = main_task(&args)?;

    // If the task we are going to do shortly involves checking out a different
    // commit, set up a restoration of the current branch
    if main_task.changes_commit() {
        final_actions.push(Action::RestoreBranch {
            name: current_branch_or_commit(args.git_root()?)?,
        });
    }

    // Now we perform the main task
    let result = match main_task {
        MainTask::PrintList { api } => print_public_items(&args, api.as_ref()),
        MainTask::PrintDiff { old_api, new_api } => print_diff(
            &args,
            old_api.as_ref(),
            new_api.as_ref(),
            &mut final_actions,
        ),
        MainTask::GenerateShellCompletionScript(shell) => {
            shell.generate(
                &mut Args::command().bin_name("cargo-public-api"),
                &mut stdout(),
            );
            Ok(())
        }
    };

    // Handle any final actions, such as checking the diff and restoring the
    // original git branch
    for action in final_actions {
        action.perform(&args)?;
    }

    result
}

fn main_task(args: &Args) -> Result<MainTask> {
    match &args.subcommand {
        Some(Subcommand::Diff(diff_args)) => main_task_from_diff_args(args, diff_args),
        Some(Subcommand::Completions { shell }) => {
            Ok(MainTask::GenerateShellCompletionScript(*shell))
        }
        None => Ok(main_task_from_args(args)),
    }
}

fn main_task_from_args(args: &Args) -> MainTask {
    if let Some(rustdoc_json) = &args.rustdoc_json {
        MainTask::print_list(RustdocJson::new(rustdoc_json.into()).boxed())
    } else {
        MainTask::print_list(CurrentDir.boxed())
    }
}

fn arg_to_api_source(arg: &str) -> Result<Box<dyn ApiSource>> {
    if is_json_file(arg) {
        Ok(RustdocJson::new(arg.into()).boxed())
    } else if semver::Version::parse(arg).is_ok() {
        Ok(PublishedCrate::new(arg).boxed())
    } else {
        bail!("Use `ref1..ref2` syntax to diff git commits");
    }
}

fn main_task_from_diff_args(args: &Args, diff_args: &DiffArgs) -> Result<MainTask> {
    let first_arg = diff_args.args.get(0);
    let second_arg = diff_args.args.get(1);
    if diff_args.args.is_empty() {
        bail!(
            "Must specify what to diff.

Examples:

Diff against a specific version of the crate published to crates.io:

    cargo public-api diff 1.2.3

Diff between two git commits:

    cargo public-api diff v0.2.0..v0.3.0

To select a package in a workspace, use the --package flag:

    cargo public-api --package my-package diff ...

See

    cargo public-api diff --help

for more.
"
        );
    } else if diff_args.args.len() > 2 {
        bail!(
            "Expected 1 or 2 arguments, but got {}",
            diff_args.args.len()
        )
    }

    let main_task = match (first_arg, second_arg) {
        (Some(first), None) if first.contains("...") => {
            bail!("Invalid git diff syntax: {first}. Use: rev1..rev2");
        }
        (Some(first), None) if first.contains("..") => {
            let commits: Vec<_> = first.split("..").collect();
            if commits.len() != 2 {
                bail!("Expected 2 commits, but got {}", commits.len());
            }
            MainTask::print_diff(
                Commit::new(args, commits[0])?.boxed(),
                Commit::new(args, commits[1])?.boxed(),
            )
        }
        (Some(first), None)
            if semver::Version::parse(first).is_ok() || first == LATEST_VERSION_ARG =>
        {
            MainTask::print_diff(PublishedCrate::new(first).boxed(), CurrentDir.boxed())
        }
        (Some(first), None) => {
            bail!("Invalid published crate version syntax: {first}");
        }
        (Some(first), Some(second)) => {
            MainTask::print_diff(arg_to_api_source(first)?, arg_to_api_source(second)?)
        }
        _ => unreachable!("We should never get here"),
    };

    Ok(main_task)
}

/// We were requested to deny diffs, so make sure there is no diff
fn check_diff(deny: &[DenyMethod], diff: &PublicApiDiff) -> Result<()> {
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

fn print_public_items(args: &Args, public_api: &dyn ApiSource) -> Result<()> {
    Plain::print_items(&mut stdout(), args, public_api.obtain_api(args)?.items())?;

    Ok(())
}

fn print_diff(
    args: &Args,
    old: &dyn ApiSource,
    new: &dyn ApiSource,
    final_actions: &mut Vec<Action>,
) -> Result<()> {
    fn check_diff(deny: &[DenyMethod], diff: PublicApiDiff) -> Action {
        Action::CheckDiff {
            diff,
            deny: deny.to_owned(),
        }
    }

    let old = old.obtain_api(args)?;
    let new = new.obtain_api(args)?;
    let diff = PublicApiDiff::between(old, new);

    Plain::print_diff(&mut stdout(), args, &diff)?;

    if let Some(Some(deny)) = args.diff_args().map(|a| &a.deny) {
        final_actions.push(check_diff(deny, diff));
    }

    Ok(())
}

impl MainTask {
    fn print_list(api: Box<dyn ApiSource>) -> MainTask {
        Self::PrintList { api }
    }

    fn print_diff(old_api: Box<dyn ApiSource>, new_api: Box<dyn ApiSource>) -> Self {
        Self::PrintDiff { old_api, new_api }
    }

    fn changes_commit(&self) -> bool {
        match self {
            MainTask::PrintDiff { old_api, new_api } => {
                old_api.changes_commit() || new_api.changes_commit()
            }
            MainTask::PrintList { api } => api.changes_commit(),
            MainTask::GenerateShellCompletionScript(_) => false,
        }
    }
}

impl Action {
    fn perform(&self, args: &Args) -> Result<()> {
        match self {
            Action::CheckDiff { deny, diff } => {
                check_diff(deny, diff)?;
            }
            Action::RestoreBranch { name } => {
                git_checkout(args, name)?;
            }
        };
        Ok(())
    }
}

impl Args {
    fn omit_blanket_impls(&self) -> bool {
        self.omits(Omit::BlanketImpls)
    }

    fn omit_auto_trait_impls(&self) -> bool {
        self.omits(Omit::AutoTraitImpls)
    }

    fn omit_auto_derived_impls(&self) -> bool {
        self.omits(Omit::AutoDerivedImpls)
    }

    fn omits(&self, to_omit: Omit) -> bool {
        self.omit.iter().flatten().any(|o| *o == to_omit)
    }

    fn git_root(&self) -> Result<PathBuf> {
        git_utils::git_root_from_manifest_path(self.manifest_path.as_path())
    }

    fn diff_args(&self) -> Option<&DiffArgs> {
        match &self.subcommand {
            Some(Subcommand::Diff(diff_args)) => Some(diff_args),
            _ => None,
        }
    }
}

/// Get CLI args via `clap` while also handling when we are invoked as a cargo
/// subcommand. When the user runs `cargo public-api -a -b -c` our args will be
/// `cargo-public-api public-api -a -b -c`.
///
/// Note that we also want to support the binary being installed with a
/// non-standard name such as `~/.cargo/bin/cargo-public-api-v0.13.0`. So we
/// can't assume the bin name is `cargo-public-api`.
fn get_args() -> Args {
    let subcommand_name = subcommand_name(std::env::args_os().next().unwrap());
    let args_os = std::env::args_os()
        .enumerate()
        .filter(|(index, arg)| *index != 1 || Some(arg) != subcommand_name.as_ref())
        .map(|(_, arg)| arg);

    let mut args = Args::parse_from(args_os);
    resolve_toolchain(&mut args);
    resolve_simplified(&mut args);
    args
}

/// Strips the `cargo-` prefix from the bin name as well as any extension. For
/// example, `cargo-public-api` becomes `public-api` and
/// `some/path/cargo-public-api-renamed.exe` becomes `public-api-renamed`.
fn subcommand_name(bin: OsString) -> Option<OsString> {
    Some(
        PathBuf::from(bin)
            .file_name()?
            .to_owned()
            .to_string_lossy()
            .strip_prefix("cargo-")?
            .strip_suffix(std::env::consts::EXE_SUFFIX)?
            .to_owned()
            .into(),
    )
}

/// Check if using a stable compiler, and use nightly if it is.
fn resolve_toolchain(args: &mut Args) {
    if toolchain::is_probably_stable(args.toolchain.as_deref()) {
        if let Some(toolchain) = args.toolchain.clone().or_else(toolchain::from_rustup) {
            eprintln!("Warning: using the `{toolchain}` toolchain for gathering the public api is not possible, switching to `nightly`");
        }
        args.toolchain = Some("nightly".to_owned());
    }
}

fn resolve_simplified(args: &mut Args) {
    if args.simplified > 0 {
        args.omit
            .get_or_insert_with(Vec::new)
            .extend([Omit::BlanketImpls, Omit::AutoTraitImpls]);
    }
    if args.simplified > 1 {
        args.omit
            .get_or_insert_with(Vec::new)
            .push(Omit::AutoDerivedImpls);
    }
}

fn is_json_file(file_name: impl AsRef<str>) -> bool {
    Path::extension(Path::new(file_name.as_ref())).map_or(false, |a| a.eq_ignore_ascii_case("json"))
}

/// Helper to reduce code duplication. We can't add [`Args`] to
/// [`git_utils::git_checkout()`] itself, because it is used in contexts where
/// [`Args`] is not available (namely in tests).
fn git_checkout(args: &Args, commit: &str) -> Result<()> {
    git_utils::git_checkout(
        &args.git_root()?,
        commit,
        !args.verbose,
        args.diff_args()
            .map(|diff_args| diff_args.force)
            .unwrap_or_default(),
    )
}

// Validate that the toolchain does not start with a `+` character.
fn parse_toolchain(s: &str) -> Result<String, &'static str> {
    if s.starts_with('+') {
        Err("toolchain must not start with a `+`")
    } else {
        Ok(s.to_owned())
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        Args::command().debug_assert();
    }

    #[test]
    fn test_subcommand_name() {
        for test in [
            ("cargo-public-api", Some("public-api")),
            ("cargo-public-api-v0.13.0", Some("public-api-v0.13.0")),
            ("relative/path/cargo-public-api", Some("public-api")),
            ("relative/cargo-public-api.foo", Some("public-api.foo")),
            ("cargo-public-api-secondary", Some("public-api-secondary")),
            ("cargo-something-else", Some("something-else")),
            ("prefix-cargo-public-api", None),
            ("/some/abs/path/cargo-public-api", Some("public-api")),
            #[cfg(windows)]
            ("c:\\abs\\cargo-public-api-old", Some("public-api-old")),
        ] {
            assert_eq!(
                subcommand_name(OsString::from(&format!(
                    "{}{}",
                    test.0,
                    std::env::consts::EXE_SUFFIX
                ))),
                test.1.map(OsString::from),
                "failed to parse \"{}\"",
                test.0,
            );
        }
    }
}
