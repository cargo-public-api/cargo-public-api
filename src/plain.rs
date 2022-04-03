use std::io::{Result, Write};

use public_items::{diff::PublicItemsDiff, PublicItem};

use ansi_term::{ANSIString, ANSIStrings, Colour};
use public_items::tokens::*;

use crate::{
    output_formatter::{print_items_with_header, OutputFormatter},
    Args,
};

pub struct Plain;

impl OutputFormatter for Plain {
    fn print_items(&self, w: &mut dyn Write, _args: &Args, items: Vec<PublicItem>) -> Result<()> {
        for item in items {
            writeln!(w, "{}", colour_item(&item))?;
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
                    writeln!(w, "{}", colour_item(item))
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
                    writeln!(w, "{}", colour_diff(changed_item))
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
                    writeln!(w, "{}", colour_item(item))
                } else {
                    writeln!(w, "+{}", item)
                }
            },
        )?;

        Ok(())
    }
}

fn colour_diff(item: &public_items::diff::ChangedPublicItem) -> String {
    let tokens = item.changed_tokens();
    let (previous, current): (Vec<ANSIString<'_>>, Vec<ANSIString<'_>>) =
        tokens.iter().map(colour_diff_token).unzip();
    format!("- {}\n+ {}", ANSIStrings(&previous), ANSIStrings(&current))
}

fn colour_diff_token(token: &ChangedToken) -> (ANSIString<'_>, ANSIString<'_>) {
    match token {
        ChangedToken::Same(token) => {
            let s = colour_item_token(token);
            let l = s.len();
            (s, Colour::White.paint(" ".repeat(l)))
        }
        ChangedToken::Inserted(token) => {
            let s = colour_item_token(token);
            (
                Colour::White.on(Colour::Green).paint(" ".repeat(s.len())),
                s,
            )
        }
        ChangedToken::Removed(token) => {
            let s = colour_item_token(token);
            let l = s.len();
            (s, Colour::White.on(Colour::Red).paint(" ".repeat(l)))
        }
    }
}

fn colour_item(item: &public_items::PublicItem) -> String {
    let styled = item
        .tokens()
        .tokens()
        .map(colour_item_token)
        .collect::<Vec<_>>();
    ANSIStrings(&styled).to_string()
}

fn colour_item_token(token: &Token) -> ANSIString<'_> {
    match token {
        Token::Symbol(text) => Colour::White.paint(text),
        Token::Qualifier(text) => Colour::Blue.paint(text),
        Token::Kind(text) => Colour::Blue.bold().paint(text),
        Token::Whitespace => Colour::White.paint(" "),
        Token::Identifier(text) => Colour::Cyan.paint(text),
        Token::Self_(text) => Colour::Blue.paint(text),
        Token::Function(text) => Colour::Yellow.paint(text),
        Token::Lifetime(text) => Colour::Blue.bold().paint(text),
        Token::Keyword(text) => Colour::Purple.paint(text),
        Token::Generic(text) => Colour::Green.bold().paint(text),
        Token::Primitive(text) => Colour::Yellow.paint(text),
        Token::Type(text) => Colour::Green.paint(text),
    }
}
