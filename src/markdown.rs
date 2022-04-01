use public_items::diff::PublicItemsDiff;

pub fn print_diff(w: &mut impl std::io::Write, diff: &PublicItemsDiff) -> std::io::Result<()> {
    if diff.removed.is_empty() && diff.changed.is_empty() && diff.added.is_empty() {
        return writeln!(w, "(No changes to the public API)");
    }

    print_items_with_header(
        w,
        "## Removed items from the public API",
        &diff.removed,
        |w, item| writeln!(w, "* {}", print_with_colour(item)),
    )?;

    print_items_with_header(
        w,
        "## Changed items in the public API",
        &diff.changed,
        |w, changed_item| writeln!(w, "{}", print_dif_with_colour(changed_item)),
    )?;

    print_items_with_header(
        w,
        "## Added items to the public API",
        &diff.added,
        |w, item| writeln!(w, "* {}", print_with_colour(item)),
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

use ansi_term::{ANSIString, ANSIStrings, Colour};
use public_items::tokens::*;

fn print_dif_with_colour(item: &public_items::diff::ChangedPublicItem) -> String {
    if let Some(tokens) = item.changed_tokens() {
        let (previous, current): (Vec<ANSIString<'_>>, Vec<ANSIString<'_>>) =
            tokens.iter().map(colour_diff).unzip();
        format!("- {}\n+ {}", ANSIStrings(&previous), ANSIStrings(&current))
    } else {
        format!(
            "* `{}` changed to\n  `{}`",
            print_with_colour(&item.old),
            print_with_colour(&item.new)
        )
    }
}

fn colour_diff(token: &ChangedToken) -> (ANSIString<'_>, ANSIString<'_>) {
    match token {
        ChangedToken::Same(token) => {
            let s = colour_token(token);
            let l = s.len();
            (s, Colour::White.paint(" ".repeat(l)))
        }
        ChangedToken::Inserted(token) => {
            let s = colour_token(token);
            (
                Colour::White.on(Colour::Green).paint(" ".repeat(s.len())),
                s,
            )
        }
        ChangedToken::Removed(token) => {
            let s = colour_token(token);
            let l = s.len();
            (s, Colour::White.on(Colour::Red).paint(" ".repeat(l)))
        }
    }
}

fn print_with_colour(item: &public_items::PublicItem) -> String {
    if let Some(tokens) = item.tokens() {
        let styled = tokens.tokens().map(colour_token).collect::<Vec<_>>();
        ANSIStrings(&styled).to_string()
    } else {
        format!("`{}`", item)
    }
}

fn colour_token(token: &Token) -> ANSIString<'_> {
    match token {
        Token::Symbol(text) => Colour::White.paint(text),
        Token::Qualifier(text) => Colour::Blue.paint(text),
        Token::Kind(text) => Colour::Blue.bold().paint(text),
        Token::Whitespace => Colour::White.paint(" "),
        Token::Identifier(text) => Colour::Cyan.paint(text),
        Token::Function(text) => Colour::Yellow.paint(text),
        Token::Lifetime(text) => Colour::Blue.bold().paint(text),
        Token::Keyword(text) => Colour::Purple.paint(text),
        Token::Generic(text) => Colour::Green.bold().paint(text),
        Token::Primitive(text) => Colour::Yellow.paint(text),
        Token::Type(text) => Colour::Green.paint(text),
    }
}
