use std::io::{Result, Write};

use public_items::{diff::PublicItemsDiff, PublicItem};

use crate::Args;

pub trait OutputFormatter {
    fn print_items(&self, w: &mut dyn Write, args: &Args, items: Vec<PublicItem>) -> Result<()>;
    fn print_diff(&self, w: &mut dyn Write, args: &Args, diff: &PublicItemsDiff) -> Result<()>;
}

pub fn print_items_with_header<T>(
    w: &mut dyn Write,
    header: &str,
    items: &[T],
    print_fn: impl Fn(&mut dyn Write, &T) -> Result<()>,
) -> Result<()> {
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
