use std::io::{Result, Write};

use public_api::{diff::PublicItemsDiff, PublicItem};

use ansi_term::{ANSIString, ANSIStrings, Color, Style};
use public_api::tokens::*;

use crate::{
    output_formatter::{print_items_with_header, OutputFormatter},
    Args,
};

pub struct Plain;

impl OutputFormatter for Plain {
    fn print_items(&self, w: &mut dyn Write, args: &Args, items: Vec<PublicItem>) -> Result<()> {
        for item in items {
            if args.color.active() {
                writeln!(w, "{}", color_item(&item))?;
            } else {
                writeln!(w, "{}", item)?;
            }
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
                    writeln!(w, "{}", color_item(item))
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
                        color_item(&changed_item.old),
                        color_item(&changed_item.new)
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
                    writeln!(w, "{}", color_item(item))
                } else {
                    writeln!(w, "+{}", item)
                }
            },
        )?;

        Ok(())
    }
}

fn color_item(item: &public_api::PublicItem) -> String {
    color_token_stream(&item.tokens, None)
}

fn color_token_stream(tokens: &TokenStream, bg: Option<Color>) -> String {
    let styled = tokens
        .tokens()
        .map(|t| color_item_token(t, bg))
        .collect::<Vec<_>>();
    ANSIStrings(&styled).to_string()
}

/// Color the given Token to render it with a nice syntax highlighting. The
/// theme is inspired by dark+ in VS Code and uses the default colors from the
/// terminal to always provide a readable and consistent color scheme.
/// An extra color can be provided to be used as background color.
fn color_item_token(token: &Token, bg: Option<Color>) -> ANSIString<'_> {
    let style = |colour: Style, text: &str| {
        if let Some(bg) = bg {
            colour.on(bg).paint(text.to_string())
        } else {
            colour.paint(text.to_string())
        }
    };
    match token {
        Token::Symbol(text) => style(Color::White.into(), text),
        Token::Qualifier(text) => style(Color::Blue.into(), text),
        Token::Kind(text) => style(Color::Blue.bold(), text),
        Token::Whitespace => style(Color::White.into(), " "),
        Token::Identifier(text) => style(Color::Cyan.into(), text),
        Token::Self_(text) => style(Color::Blue.into(), text),
        Token::Function(text) => style(Color::Yellow.into(), text),
        Token::Lifetime(text) => style(Color::Blue.bold(), text),
        Token::Keyword(text) => style(Color::Purple.into(), text),
        Token::Generic(text) => style(Color::Green.bold(), text),
        Token::Primitive(text) => style(Color::Yellow.into(), text),
        Token::Type(text) => style(Color::Green.into(), text),
    }
}
