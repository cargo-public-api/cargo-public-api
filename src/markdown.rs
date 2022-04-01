use std::io::{Result, Write};

use public_items::{diff::PublicItemsDiff, PublicItem};

use crate::{output_formatter::print_items_with_header, Args, OutputFormatter};

pub struct Markdown;

impl OutputFormatter for Markdown {
    fn print_items(&self, _w: &mut dyn Write, _args: &Args, _items: Vec<PublicItem>) -> Result<()> {
        todo!("Not yet implemented because I'm not sure what the output should look like. Feel free to open a PR with a proposal.")
    }

    fn print_diff(&self, w: &mut dyn Write, _args: &Args, diff: &PublicItemsDiff) -> Result<()> {
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
                    "* `{}` changed to\n  `{}`",
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
}
