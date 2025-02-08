# In CI we want to notice if a command is missing so always try to run the
# command if in CI.
if_command_exists_or_in_ci() {
    command="$1"

    if command -v "$command" >/dev/null; then
        return 0
    elif [ -n "${CI:-}" ]; then
        return 0
    else
        echo "INFO: Not running \`$command\` because it is not installed and we are not in CI"
        return 1
    fi
}

workspace_dir_from_name() {
    # By default this behaves as if the top level workspace is being linted, which
    # is the most relevant workspace.
    workspace="${1:-.}"

    # For clarity in CI config we allow the alias "cargo-public-api" for the
    # root workspace.
    if [ "$workspace" = "cargo-public-api" ]; then
        workspace="."
    fi

    echo "$workspace"
}
