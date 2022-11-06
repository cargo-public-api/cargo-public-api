// deny in CI, only warn here
#![warn(clippy::all, clippy::pedantic)]

use std::io::stdout;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use arg_types::{Color, DenyMethod};
use plain::Plain;
use public_api::diff::PublicApiDiff;
use public_api::{Options, PublicApi, MINIMUM_RUSTDOC_JSON_VERSION};

use clap::Parser;
use rustdoc_json::BuildError;

mod arg_types;
mod error;
mod git_utils;
mod plain;
mod published_crate;
mod toolchain;

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

    /// If `true`, items that strongly belongs to a parent (such as struct
    /// fields, enum variants, and associated functions) are rendered with one
    /// level of indentation (4 spaces). Note that the output is still flat, in
    /// the sense that e.g. contents of `mod`s are not indented.
    ///
    /// The default value is `false`, because this is only something that you
    /// typically want to do if presenting the output to a human.
    #[clap(long)]
    with_indentation: bool,

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
    /// Using the current git repo has the benefit of making it likely for the
    /// build to succeed. If we e.g. were to git clone a temporary copy of a
    /// commit ourselves, the risk is high that additional steps are needed
    /// before a build can succeed. Such as the need to set up git submodules.
    #[clap(long, min_values = 2, max_values = 2)]
    diff_git_checkouts: Option<Vec<String>>,

    /// Raise this flag to discard working tree changes during git checkouts when
    /// `--diff-git-checkouts` is used.
    #[clap(long)]
    force_git_checkouts: bool,

    /// Usage: --diff-rustdoc-json <RUSTDOC_JSON_PATH_1> <RUSTDOC_JSON_PATH_2>
    ///
    /// Diff the public API across two different rustdoc JSON files.
    #[clap(long, min_values = 2, max_values = 2)]
    diff_rustdoc_json: Option<Vec<String>>,

    /// Usage: --diff-published some-crate@1.2.3
    ///
    /// Diff the current API against the API in a published version.
    #[clap(long)]
    diff_published: Option<String>,

    /// Automatically resolves to either `--diff-git-checkouts` or
    /// `--diff-rustdoc-json` or `--diff-rustdoc-json` depending on if args ends
    /// in `.json` or not, or if they contain `@`.
    ///
    /// Examples:
    ///
    ///   cargo public-api --diff v0.2.0 v0.3.0
    ///
    /// resolves to
    ///
    ///   cargo public-api --diff-git-checkouts v0.2.0 v0.3.0
    ///
    /// but
    ///
    ///   cargo public-api --diff v0.2.0.json v0.3.0.json
    ///
    /// resolves to
    ///
    ///   cargo public-api --diff-rustdoc-json v0.2.0.json v0.3.0.json
    ///
    /// and
    ///
    ///   cargo public-api --diff some-crate@1.2.3
    ///
    /// resolves to
    ///
    ///   cargo public-api --diff-published some-crate@1.2.3
    ///
    #[clap(long, min_values = 1, max_values = 2)]
    diff: Option<Vec<String>>,

    /// Usage: --rustdoc-json <RUSTDOC_JSON_PATH>
    ///
    /// List the public API based on the given rustdoc JSON file. Try for example
    ///
    ///     rustup component add rust-docs-json --toolchain  nightly
    ///
    /// and then
    ///
    ///     cargo public-api --rustdoc-json ~/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/share/doc/rust/json/std.json
    ///
    #[clap(long)]
    rustdoc_json: Option<String>,

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
    #[clap(long, arg_enum, default_value = "auto")]
    color: Color,

    /// Show detailed info about processing. For debugging purposes. The output
    /// is not stable and can change across patch versions.
    #[clap(long, hide = true)]
    verbose: bool,

    /// Do not sort the output lexicographically. Instead keep the logical
    /// grouping where e.g. struct fields come after structs.
    #[clap(long, hide = true)]
    unsorted: bool,

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

/// This represents an action that we want to do at some point.
pub enum Action {
    /// The `--deny` arg allows the user to disallow the occurrence of API
    /// changes. We are to check that the diff is allowed.
    CheckDiff(PublicApiDiff),

    /// Doing a `--diff-git-checkouts` involves doing `git checkout`s.
    /// Afterwards, we want to restore the original branch the user was on, to
    /// not mess up their work tree.
    RestoreBranch(String),
}

fn main_() -> Result<()> {
    let args = get_args()?;

    // A list of actions to perform after we have listed or diffed. Typical
    // examples: restore a git branch or check that a diff is allowed
    let mut final_actions = vec![];

    let result = list_or_diff(&args, &mut final_actions);

    for action in final_actions {
        action.perform(&args)?;
    }

    result
}

fn list_or_diff(args: &Args, final_actions: &mut Vec<Action>) -> Result<()> {
    if let Some(commits) = &args.diff_git_checkouts {
        print_diff_between_two_commits(args, commits, final_actions)
    } else if let Some(files) = &args.diff_rustdoc_json {
        // clap ensures both args exists if we get here
        print_diff_between_two_rustdoc_json_files(
            args,
            files.get(0).unwrap(),
            files.get(1).unwrap(),
            final_actions,
        )
    } else if let Some(package_spec) = &args.diff_published {
        print_diff_between_two_rustdoc_json_files(
            args,
            &published_crate::build_rustdoc_json(package_spec, args)?,
            &rustdoc_json_for_current_dir(args)?,
            final_actions,
        )
    } else if let Some(rustdoc_json) = &args.rustdoc_json {
        print_public_items_from_json(args, rustdoc_json)
    } else {
        print_public_items_of_current_dir(args)
    }
}

fn check_diff(args: &Args, diff: &PublicApiDiff) -> Result<()> {
    match &args.deny {
        // We were requested to deny diffs, so make sure there is no diff
        Some(deny) => {
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

        // No diff related stuff to care about, all is Ok
        _ => Ok(()),
    }
}

fn print_public_items_of_current_dir(args: &Args) -> Result<()> {
    let public_api = public_api_for_current_dir(args)?;
    print_public_items(args, &public_api)
}

fn print_public_items_from_json(args: &Args, json_path: &str) -> Result<()> {
    let public_api = public_api_from_rustdoc_json_path(json_path, args)?;
    print_public_items(args, &public_api)
}

fn print_public_items(args: &Args, public_api: &PublicApi) -> Result<()> {
    Plain::print_items(&mut stdout(), args, public_api.items())?;

    Ok(())
}

fn print_diff_between_two_commits(
    args: &Args,
    commits: &[String],
    final_actions: &mut Vec<Action>,
) -> Result<()> {
    let old_commit = commits.get(0).expect("clap makes sure first commit exist");
    let new_commit = commits.get(1).expect("missing second commit!");

    // Validate provided commits and resolve relative refs like HEAD to actual commits
    let old_commit = git_utils::resolve_ref(&args.git_root()?, old_commit)?;
    let new_commit = git_utils::resolve_ref(&args.git_root()?, new_commit)?;

    // Checkout the first commit and remember the branch so we can restore it
    let original_branch = git_checkout(args, &old_commit)?;
    let old = public_api_for_current_dir(args)?;
    final_actions.push(Action::RestoreBranch(original_branch));

    // Checkout the second commit
    git_checkout(args, &new_commit)?;
    let new = public_api_for_current_dir(args)?;

    // Calculate the diff
    print_diff(args, old, new, final_actions)?;

    Ok(())
}

fn print_diff_between_two_rustdoc_json_files(
    args: &Args,
    old_file: impl AsRef<Path>,
    new_file: impl AsRef<Path>,
    final_actions: &mut Vec<Action>,
) -> Result<()> {
    let old = public_api_from_rustdoc_json_path(old_file, args)?;
    let new = public_api_from_rustdoc_json_path(new_file, args)?;

    print_diff(args, old, new, final_actions)?;

    Ok(())
}

fn print_diff(
    args: &Args,
    old: PublicApi,
    new: PublicApi,
    final_actions: &mut Vec<Action>,
) -> Result<()> {
    let diff = PublicApiDiff::between(old, new);

    Plain::print_diff(&mut stdout(), args, &diff)?;

    final_actions.push(Action::CheckDiff(diff));

    Ok(())
}

impl Action {
    fn perform(&self, args: &Args) -> Result<()> {
        match self {
            Action::CheckDiff(diff) => {
                check_diff(args, diff)?;
            }
            Action::RestoreBranch(name) => {
                git_checkout(args, name)?;
            }
        };
        Ok(())
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
fn get_args() -> Result<Args> {
    let args_os = std::env::args_os()
        .enumerate()
        .filter(|(index, arg)| *index != 1 || arg != "public-api")
        .map(|(_, arg)| arg);

    let mut args = Args::parse_from(args_os);
    resolve_diff_shorthand(&mut args);
    resolve_toolchain(&mut args);

    // Manually check this until a `cargo public-api diff ...` subcommand is in
    // place, which will enable clap to perform this check
    if args.deny.is_some()
        && args.diff_git_checkouts.is_none()
        && args.diff_published.is_none()
        && args.diff_rustdoc_json.is_none()
    {
        Err(anyhow!("`--deny` can only be used when diffing"))
    } else {
        Ok(args)
    }
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

/// Resolve `--diff` to either `--diff-git-checkouts` or `--diff-rustdoc-json`
fn resolve_diff_shorthand(args: &mut Args) {
    if let Some(diff_args) = args.diff.clone() {
        fn is_json_file(file_name: &String) -> bool {
            Path::extension(Path::new(file_name)).map_or(false, |a| a.eq_ignore_ascii_case("json"))
        }

        if diff_args.iter().all(is_json_file) {
            args.diff_rustdoc_json = Some(diff_args);
        } else if diff_args.iter().any(|a| a.contains('@')) {
            args.diff_published = diff_args.first().cloned();
        } else {
            args.diff_git_checkouts = Some(diff_args);
        }
    }
}

/// Figure out what [`Options`] to pass to
/// [`public_api::PublicApi::from_rustdoc_json_str`] based on our
/// [`Args`]
fn get_options(args: &Args) -> Options {
    let mut options = Options::default();
    options.with_blanket_implementations = args.with_blanket_implementations;
    options.with_indentation = args.with_indentation;
    options.sorted = !args.unsorted;
    options
}

/// Helper to reduce code duplication. We can't add [`Args`] to
/// [`git_utils::git_checkout()`] itself, because it is used in contexts where
/// [`Args`] is not available (namely in tests).
fn git_checkout(args: &Args, commit: &str) -> Result<String> {
    git_utils::git_checkout(
        commit,
        &args.git_root()?,
        !args.verbose,
        args.force_git_checkouts,
    )
}

/// Builds the public API for the library in the current working directory. Note
/// that we sometimes checkout a different commit before invoking this function,
/// which means it will return the public API of that commit.
fn public_api_for_current_dir(args: &Args) -> Result<PublicApi, anyhow::Error> {
    let json_path = rustdoc_json_for_current_dir(args)?;
    public_api_from_rustdoc_json_path(json_path, args)
}

/// Builds the rustdoc JSON for the library in the current working directory.
/// Also see [`public_api_for_current_dir()`].
fn rustdoc_json_for_current_dir(args: &Args) -> Result<PathBuf, anyhow::Error> {
    let builder = builder_from_args(args);
    build_rustdoc_json(builder)
}

/// Creates a rustdoc JSON builder based on the args to this program.
fn builder_from_args(args: &Args) -> rustdoc_json::Builder {
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
    builder
}

/// Helper to build rustdoc JSON with a builder while also handling any virtual
/// manifest errors.
fn build_rustdoc_json(builder: rustdoc_json::Builder) -> Result<PathBuf, anyhow::Error> {
    match builder.build() {
        Err(BuildError::VirtualManifest(manifest_path)) => virtual_manifest_error(&manifest_path),
        res => Ok(res?),
    }
}

fn public_api_from_rustdoc_json_path<T: AsRef<Path>>(
    json_path: T,
    args: &Args,
) -> Result<PublicApi> {
    let options = get_options(args);

    let rustdoc_json = &std::fs::read_to_string(&json_path)
        .with_context(|| format!("Failed to read rustdoc JSON at {:?}", json_path.as_ref()))?;

    if args.verbose {
        println!("Processing {:?}", json_path.as_ref());
    }

    let public_api = PublicApi::from_rustdoc_json_str(rustdoc_json, options).with_context(|| {
        format!(
            "Failed to parse rustdoc JSON at {:?}.\n\
            This version of `cargo public-api` requires at least:\n\n    {}\n\n\
            If you have that, it might be `cargo public-api` that is out of date. Try\n\
            to install the latest version with `cargo install cargo-public-api`. If the\n\
            issue remains, please report at\n\n    https://github.com/Enselic/cargo-public-api/issues",
            json_path.as_ref(),
            MINIMUM_RUSTDOC_JSON_VERSION,
        )
    })?;

    if args.verbose {
        public_api.missing_item_ids().for_each(|i| {
            println!("NOTE: rustdoc JSON missing referenced item with ID \"{i}\"");
        });
    }

    Ok(public_api)
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
