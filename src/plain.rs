use std::io::{Result, Write};

use public_items::{diff::PublicItemsDiff, PublicItem};

use ansi_term::{ANSIString, ANSIStrings, Colour, Style};
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
                    writeln!(w, "-{}", colour_item(item))
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
                    writeln!(w, "+{}", colour_item(item))
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
    let (previous, current): (Vec<String>, Vec<String>) =
        tokens.iter().map(colour_diff_token).unzip();
    format!("- {}\n+ {}", previous.join(""), current.join(""))
}

fn colour_diff_token(token: &ChangedTokenStream) -> (String, String) {
    match token {
        ChangedTokenStream::Same(tokens) => {
            let s = colour_token_stream(tokens, None);
            (s, " ".repeat(tokens.tokens_len()))
        }
        ChangedTokenStream::Changed { removed, inserted } => {
            let removed_len = removed.tokens_len();
            let inserted_len = inserted.tokens_len();
            let l = removed_len.max(inserted_len);
            let mut removed = removed.clone();
            let mut inserted = inserted.clone();
            removed.extend(vec![Token::Whitespace; l - removed_len]);
            inserted.extend(vec![Token::Whitespace; l - inserted_len]);

            (
                colour_token_stream(&removed, Some(Colour::Red)),
                colour_token_stream(&inserted, Some(Colour::Green)),
            )
        }
    }
}

fn colour_item(item: &public_items::PublicItem) -> String {
    colour_token_stream(&item.tokens, None)
}

fn colour_token_stream(tokens: &TokenStream, bg: Option<Colour>) -> String {
    let styled = tokens
        .tokens()
        .map(|t| colour_item_token(t, bg))
        .collect::<Vec<_>>();
    ANSIStrings(&styled).to_string()
}

fn colour_item_token(token: &Token, bg: Option<Colour>) -> ANSIString<'_> {
    let style = |colour: Style, text: &str| {
        if let Some(overrule) = bg {
            colour.on(overrule).paint(text.to_string())
        } else {
            colour.paint(text.to_string())
        }
    };
    match token {
        Token::Symbol(text) => style(Colour::White.into(), text),
        Token::Qualifier(text) => style(Colour::Blue.into(), text),
        Token::Kind(text) => style(Colour::Blue.bold(), text),
        Token::Whitespace => style(Colour::White.into(), " "),
        Token::Identifier(text) => style(Colour::Cyan.into(), text),
        Token::Self_(text) => style(Colour::Blue.into(), text),
        Token::Function(text) => style(Colour::Yellow.into(), text),
        Token::Lifetime(text) => style(Colour::Blue.bold(), text),
        Token::Keyword(text) => style(Colour::Purple.into(), text),
        Token::Generic(text) => style(Colour::Green.bold(), text),
        Token::Primitive(text) => style(Colour::Yellow.into(), text),
        Token::Type(text) => style(Colour::Green.into(), text),
    }
}
