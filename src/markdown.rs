use public_items::diff::PublicItemsDiff;

pub fn print_diff(w: &mut impl std::io::Write, diff: &PublicItemsDiff) -> std::io::Result<()> {
    if diff.removed.is_empty() && diff.changed.is_empty() && diff.added.is_empty() {
        return writeln!(w, "(No changes to the public API)");
    }

    print_items_with_header(
        w,
        "## Removed items from the public API",
        &diff.removed,
        |w, item| writeln!(w, "* `{}`", item),
    )?;

    print_items_with_header(
        w,
        "## Changed items in the public API",
        &diff.changed,
        |w, changed_item| {
            writeln!(
                w,
                "* `{}` became <br/> `{}`",
                changed_item.old, changed_item.new
            )
        },
    )?;

    print_items_with_header(
        w,
        "## Added items to the public API",
        &diff.added,
        |w, item| writeln!(w, "* `{}`", item),
    )?;

    Ok(())
}

fn print_items_with_header<W: std::io::Write, T>(
    w: &mut W,
    header: &str,
    items: &[T],
    print_fn: impl Fn(&mut W, &T) -> std::io::Result<()>,
) -> std::io::Result<()> {
    writeln!(w, "{}", header)?;
    if items.is_empty() {
        writeln!(w, "(none)")?;
    } else {
        for item in items {
            print_fn(w, item)?;
        }
    }
    writeln!(w)
}
