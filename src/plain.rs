use std::io::{Result, Write};

use public_api::{diff::PublicItemsDiff, PublicItem};

use ansi_term::Color;

use crate::{
    output_formatter::{print_items_with_header, OutputFormatter},
    Args,
};

pub struct Plain;

impl OutputFormatter for Plain {
    fn print_items(&self, w: &mut dyn Write, _args: &Args, items: Vec<PublicItem>) -> Result<()> {
        for item in items {
            writeln!(w, "{}", item)?;
        }

        Ok(())
    }

    fn print_diff(&self, w: &mut dyn Write, args: &Args, diff: &PublicItemsDiff) -> Result<()> {
        let use_color = args.color.active();

        print_items_with_header(
            w,
            "Removed items from the public API\n\
             =================================",
            &diff.removed,
            |w, item| {
                if use_color {
                    writeln!(w, "{}", Color::Red.paint(item.to_string()))
                } else {
                    writeln!(w, "-{}", item)
                }
            },
        )?;

        print_items_with_header(
            w,
            "Changed items in the public API\n\
             ===============================",
            &diff.changed,
            |w, changed_item| {
                if use_color {
                    writeln!(
                        w,
                        "{}\n{}",
                        Color::Red.paint(changed_item.old.to_string()),
                        Color::Green.paint(changed_item.new.to_string())
                    )
                } else {
                    writeln!(w, "-{}\n+{}", changed_item.old, changed_item.new)
                }
            },
        )?;

        print_items_with_header(
            w,
            "Added items to the public API\n\
             =============================",
            &diff.added,
            |w, item| {
                if use_color {
                    writeln!(w, "{}", Color::Green.paint(item.to_string()))
                } else {
                    writeln!(w, "+{}", item)
                }
            },
        )?;

        Ok(())
    }
}
