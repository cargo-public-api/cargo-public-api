// deny in CI, only warn here
#![warn(clippy::all)]

use std::ffi::OsString;
use std::io::{stderr, stdout};
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
mod vendor;

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "List and diff the public API of Rust library crates between releases and commits.",
    long_about = "List and diff the public API of Rust library crates between releases and commits. Website: https://github.com/cargo-public-api/cargo-public-api",
    bin_name = "cargo public-api"
)]
#[command(flatten_help = true)]
pub struct Args {
    /// Path to `Cargo.toml`.
    #[arg(global = true, long, value_name = "PATH", default_value = "Cargo.toml")]
    manifest_path: PathBuf,

    /// Name of package in workspace to list or diff the public API for.
    #[arg(global = true, long, short)]
    package: Option<String>,

    /// Omit noisy items. Can be used more than once.
    ///
    /// | Usage | Corresponds to                                           |
    /// |-------|----------------------------------------------------------|
    /// | -s    | --omit blanket-impls                                     |
    /// | -ss   | --omit blanket-impls,auto-trait-impls                    |
    /// | -sss  | --omit blanket-impls,auto-trait-impls,auto-derived-impls |
    #[clap(verbatim_doc_comment)]
    #[arg(global = true, short, long, action = clap::ArgAction::Count)]
    simplified: u8,

    /// Omit specified items.
    #[arg(global = true, long, value_enum, value_delimiter = ',')]
    omit: Option<Vec<Omit>>,

    /// Space or comma separated list of features to activate
    #[arg(global = true, long, short = 'F')]
    features: Vec<String>,

    /// Activate all available features
    #[arg(global = true, long)]
    all_features: bool,

    /// Do not activate the `default` feature
    #[arg(global = true, long)]
    no_default_features: bool,

    /// Build for the target triple
    #[arg(global = true, long)]
    target: Option<String>,

    /// When to color the output.
    ///
    /// By default, `--color=auto` is active. Using just `--color` without an
    /// arg is equivalent to `--color=always`.
    #[arg(global = true, long, value_enum)]
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
    #[arg(global = true, long, value_name = "RUSTDOC_JSON_PATH", hide = true)]
    rustdoc_json: Option<String>,

    /// Show detailed info about processing.
    ///
    /// For debugging purposes. The output is not stable and can change across
    /// patch versions.
    #[arg(global = true, long, hide = true)]
    verbose: bool,

    /// Show the hidden "sorting prefix" that makes items nicely grouped
    ///
    /// Only intended for debugging this tool.
    #[arg(global = true, long, hide = true)]
    debug_sorting: bool,

    /// Where to put rustdoc JSON build artifacts.
    ///
    /// Hidden by default because it will typically not be needed by users.
    /// Mainly useful to allow tests to run in parallel.
    #[arg(global = true, long, value_name = "PATH", hide = true)]
    target_dir: Option<PathBuf>,

    /// Forwarded to rustdoc JSON build command
    #[arg(global = true, long, hide = true)]
    cap_lints: Option<String>,

    #[command(subcommand)]
    subcommand: Option<Subcommand>,
}

/// We don't want `toolchain` in [Args] because we only support the `cargo
/// +toolchain public-api` way of picking toolchain. But we still want to
/// resolve the toolchain to use once and pass it around a bit. This helper
/// structs solves this for us.
#[derive(Debug)]
struct ArgsAndToolchain {
    args: Args,
    toolchain: Option<String>,
}

/// The subcommand used for diffing.
#[derive(Parser, Debug)]
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

    #[clap(verbatim_doc_comment)]
    /// What to diff.
    ///
    /// EXAMPLES
    /// ========
    ///
    /// Diff current working tree version of `specific-crate` against published version 1.2.3:
    ///
    ///     cargo public-api -p specific-crate diff 1.2.3
    ///
    /// Diff between commits:
    ///
    ///     cargo public-api diff v0.2.0..v0.3.0
    ///
    /// See
    ///
    ///     cargo public-api diff --help
    ///
    /// for more examples and more info.
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
    /// Diff a published version of a crate against the current working tree:
    ///
    ///     cargo public-api diff 1.2.3
    ///
    /// Diff the latest version of a crate against the current working tree:
    ///
    ///     cargo public-api diff latest
    ///
    /// Diff between two published versions of any crate:
    ///
    ///     cargo public-api -p example_api diff 0.1.0 0.2.0
    ///
    /// Diff current working tree version of `specific-crate` against published version 1.2.3:
    ///
    ///     cargo public-api -p specific-crate diff 1.2.3
    ///
    /// Diff between commits:
    ///
    ///     cargo public-api diff v0.2.0..v0.3.0
    ///
    /// Diff between rustdoc JSON files:
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
    ///    --color                -- When to color the output
    ///    --debug-sorting        -- Show the hidden "sorting prefix" that makes items nicely grouped
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
    // We use the same underlying tracing library as the Rust compiler. Run with
    // the env var `RUST_LOG` set to e.g. `debug` to get started.
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_writer(stderr) // See https://github.com/tokio-rs/tracing/issues/2492
        .init();

    let argst = get_args();

    // A list of actions to perform after we have listed or diffed. Typical
    // examples: restore a git branch or check that a diff is allowed
    let mut final_actions = vec![];

    // Now figure out our main task. Typically listing or diffing the public API
    let main_task = main_task(&argst.args)?;

    // If the task we are going to do shortly involves checking out a different
    // commit, set up a restoration of the current branch
    if main_task.changes_commit() {
        final_actions.push(Action::RestoreBranch {
            name: current_branch_or_commit(argst.args.git_root()?)?,
        });
    }

    // Now we perform the main task
    let result = match main_task {
        MainTask::PrintList { api } => print_public_items(&argst, api.as_ref()),
        MainTask::PrintDiff { old_api, new_api } => print_diff(
            &argst,
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
        action.perform(&argst.args)?;
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

fn arg_to_api_source(arg: Option<&str>) -> Result<Box<dyn ApiSource>> {
    match arg {
        Some(arg) if is_json_file(arg) => Ok(RustdocJson::new(arg.into()).boxed()),
        Some(arg) if semver::Version::parse(arg).is_ok() => {
            Ok(PublishedCrate::new(Some(arg)).boxed())
        }
        None => Ok(PublishedCrate::new(None).boxed()),
        _ => bail!("Use `ref1..ref2` syntax to diff git commits"),
    }
}

fn main_task_from_diff_args(args: &Args, diff_args: &DiffArgs) -> Result<MainTask> {
    if diff_args.args.len() > 2 {
        bail!(
            "Expected 1 or 2 arguments, but got {}",
            diff_args.args.len()
        )
    }

    let first_arg = diff_args.args.first();
    let second_arg = diff_args.args.get(1);

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
            MainTask::print_diff(PublishedCrate::new(Some(first)).boxed(), CurrentDir.boxed())
        }
        (Some(first), Some(second)) => MainTask::print_diff(
            arg_to_api_source(Some(first))?,
            arg_to_api_source(Some(second))?,
        ),
        (None, _) => MainTask::print_diff(PublishedCrate::new(None).boxed(), CurrentDir.boxed()),
        (Some(first), None) => {
            bail!("Invalid published crate version syntax: {first}");
        }
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

fn print_public_items(argst: &ArgsAndToolchain, public_api: &dyn ApiSource) -> Result<()> {
    Plain::print_items(
        &mut stdout(),
        &argst.args,
        public_api.obtain_api(argst)?.items(),
    )?;

    Ok(())
}

fn print_diff(
    argst: &ArgsAndToolchain,
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

    let old = old.obtain_api(argst)?;
    let new = new.obtain_api(argst)?;
    let diff = PublicApiDiff::between(old, new);

    Plain::print_diff(&mut stdout(), &argst.args, &diff)?;

    if let Some(Some(deny)) = argst.args.diff_args().map(|a| &a.deny) {
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
fn get_args() -> ArgsAndToolchain {
    let subcommand_name = subcommand_name(std::env::args_os().next().unwrap());
    let args_os = std::env::args_os()
        .enumerate()
        .filter(|(index, arg)| *index != 1 || Some(arg) != subcommand_name.as_ref())
        .map(|(_, arg)| arg);

    let mut args = Args::parse_from(args_os);
    resolve_simplified(&mut args);
    resolve_toolchain(args)
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
fn resolve_toolchain(args: Args) -> ArgsAndToolchain {
    let toolchain = if toolchain::is_probably_stable() {
        if let Some(toolchain) = toolchain::from_rustup() {
            eprintln!("Warning: using the `{toolchain}` toolchain for gathering the public api is not possible, switching to `nightly`");
        }
        Some("nightly".to_owned())
    } else {
        None
    };

    ArgsAndToolchain { args, toolchain }
}

/// Translates `--simplified` into `--omit` args.
///
/// | number of `--simplified` args | corresponds to `--omit` of                          |
/// |-------------------------------|-----------------------------------------------------|
/// | 1                             | `blanket-impls`                                     |
/// | 2                             | `blanket-impls,auto-trait-impls`                    |
/// | 3                             | `blanket-impls,auto-trait-impls,auto-derived-impls` |
fn resolve_simplified(args: &mut Args) {
    if args.simplified > 0 {
        let omit = args.omit.get_or_insert_with(Vec::new);
        omit.push(Omit::BlanketImpls);

        if args.simplified > 1 {
            omit.push(Omit::AutoTraitImpls);
        }

        if args.simplified > 2 {
            omit.push(Omit::AutoDerivedImpls);
        }
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
