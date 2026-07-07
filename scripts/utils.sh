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

# Tools like 'cargo audit' tend to give too many false positives for us. Don't
# run them in Nightly CI to avoid annoying flakiness. Still run them for PRs and
# releases however.
if_command_exists_or_in_ci_but_not_nightly() {
    command="$1"

    if [ "${NIGHTLY_CI:-}" = "true" ]; then
        echo "INFO: Not running \`$command\` because NIGHTLY_CI=true"
        return 1
    fi

    if_command_exists_or_in_ci "$command"
}
