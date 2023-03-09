# Do not run this script directly, it is meant to be run via a `cargo test`

extra_fpath="$1"
if [ -z "${extra_fpath}" ]; then
    echo "Usage: zsh -f $0 <fpath_with_completion_scripts>"
    exit 1
fi

# Create a pseudo-terminal to test with. Make it non-blocking with `-b` to
# prevent `zpty -r` from blocking forever.
zmodload zsh/zpty
zpty -b testable-zsh zsh -f

# We want to test the zsh completion system, so init it. `compinit -U` means "do
# not create a compdump file" and `-u` stops `compinit` from being paranoid
# about the security of our fpath.
zpty -w testable-zsh "fpath+=${extra_fpath}"
zpty -w testable-zsh "autoload -U compinit && compinit -D -u"

# Now trigger auto-completion
zpty -n -w testable-zsh "cargo public-api --"$'\t'

# Create a file to put the pseudio-terminal output in.
output_file=$(mktemp)
trap "rm -f ${output_file}" EXIT

# Poll the output until we find what we want (or timeout).
start=$(date +%s)
timeout=59
expected="--simplified"
while ! grep -e "${expected}" "${output_file}" ; do
    zpty -r testable-zsh >> "${output_file}" # Non-blocking because of `-b` above.
    if (( $(date +%s) - start > timeout )); then
        echo "FAIL: Timed out after ${timeout} seconds waiting for \`${expected}\` to appear in auto-completion proposals"
        echo "Output from pseudo-terminal was:"
        cat "${output_file}" | strings
        exit 1
    fi
done
