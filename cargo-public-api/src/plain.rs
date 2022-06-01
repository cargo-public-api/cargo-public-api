use std::io::{Result, Write};

use ansi_term::{ANSIString, ANSIStrings, Color, Style};
use public_api::{diff::PublicItemsDiff, tokens::Token, PublicItem};

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
                    writeln!(w, "-{}", color_item(item))
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
                    let old_tokens: Vec<&Token> = changed_item.old.tokens().collect();
                    let new_tokens: Vec<&Token> = changed_item.new.tokens().collect();
                    let diff_slice = diff::slice(old_tokens.as_slice(), new_tokens.as_slice());
                    writeln!(
                        w,
                        "-{}\n+{}",
                        color_item_with_diff(&changed_item.old, &diff_slice, true),
                        color_item_with_diff(&changed_item.new, &diff_slice, false),
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
                    writeln!(w, "+{}", color_item(item))
                } else {
                    writeln!(w, "+{}", item)
                }
            },
        )?;

        Ok(())
    }
}

fn color_item(item: &public_api::PublicItem) -> String {
    color_token_stream(item.tokens(), None)
}

fn color_token_stream<'a>(tokens: impl Iterator<Item = &'a Token>, bg: Option<Color>) -> String {
    let styled = tokens.map(|t| color_item_token(t, bg)).collect::<Vec<_>>();
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
    #[allow(clippy::match_same_arms)]
    match token {
        Token::Symbol(text) => style(Style::default(), text),
        Token::Qualifier(text) => style(Color::Blue.into(), text),
        Token::Kind(text) => style(Color::Blue.into(), text),
        Token::Whitespace => style(Style::default(), " "),
        Token::Identifier(text) => style(Color::Cyan.into(), text),
        Token::Self_(text) => style(Color::Blue.into(), text),
        Token::Function(text) => style(Color::Yellow.into(), text),
        Token::Lifetime(text) => style(Color::Blue.into(), text),
        Token::Keyword(text) => style(Color::Blue.into(), text),
        Token::Generic(text) => style(Color::Green.into(), text),
        Token::Primitive(text) => style(Color::Green.into(), text),
        Token::Type(text) => style(Color::Green.into(), text),
    }
}

/// Returns a styled string similar to `color_item_token`, but where whole tokens are highlighted if
/// they contain a difference.
fn color_item_with_diff(
    item: &PublicItem,
    diff_slice: &[diff::Result<&&Token>],
    is_old_item: bool,
) -> String {
    let diff_iter = diff_slice
        .iter()
        .filter_map(|diff_result| match diff_result {
            diff::Result::Left(&token) => {
                if is_old_item {
                    let style = Some(Color::Fixed(9).on(Color::Fixed(52)).bold());
                    Some((style, token))
                } else {
                    None
                }
            }
            diff::Result::Both(&token, _) => Some((None, token)),
            diff::Result::Right(&token) => {
                if is_old_item {
                    None
                } else {
                    let style = Some(Color::Fixed(10).on(Color::Fixed(22)).bold());
                    Some((style, token))
                }
            }
        });
    let styled_strings = item
        .tokens()
        .zip(diff_iter)
        .map(|(token, (maybe_diff_style, diff_token))| {
            debug_assert_eq!(token, diff_token);
            if let Some(diff_style) = maybe_diff_style {
                diff_style.paint(token.text())
            } else {
                color_item_token(token, None)
            }
        })
        .collect::<Vec<_>>();

    ANSIStrings(&styled_strings).to_string()
}
