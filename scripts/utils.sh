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
