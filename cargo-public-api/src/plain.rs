use std::io::{Result, Write};

use nu_ansi_term::{AnsiString, AnsiStrings, Color, Style};
use public_api::{diff::PublicApiDiff, tokens::Token, PublicItem};

use crate::Args;

pub struct Plain;

impl Plain {
    pub fn print_items(w: &mut dyn Write, args: &Args, items: &[PublicItem]) -> Result<()> {
        for item in items {
            print_item(args, w, item)?;
        }

        Ok(())
    }

    pub fn print_diff(w: &mut dyn Write, args: &Args, diff: &PublicApiDiff) -> Result<()> {
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
                        color_item_with_diff(&diff_slice, true),
                        color_item_with_diff(&diff_slice, false),
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

fn print_item(args: &Args, w: &mut dyn Write, item: &PublicItem) -> Result<()> {
    if args.color.active() {
        writeln!(w, "{}", color_item(item))
    } else {
        writeln!(w, "{}", item)
    }
}

fn color_item(item: &public_api::PublicItem) -> String {
    color_token_stream(item.tokens(), None)
}

fn color_token_stream<'a>(tokens: impl Iterator<Item = &'a Token>, bg: Option<Color>) -> String {
    let styled = tokens.map(|t| color_item_token(t, bg)).collect::<Vec<_>>();
    AnsiStrings(&styled).to_string()
}

/// Color the given Token to render it with a nice syntax highlighting. The
/// theme is inspired by dark+ in VS Code and uses the default colors from the
/// terminal to always provide a readable and consistent color scheme.
/// An extra color can be provided to be used as background color.
fn color_item_token(token: &Token, bg: Option<Color>) -> AnsiString<'_> {
    let style = |colour: Style, text: &str| {
        bg.map_or_else(
            || colour.paint(text.to_string()),
            |bg| colour.on(bg).paint(text.to_string()),
        )
    };
    #[allow(clippy::match_same_arms)]
    match token {
        Token::Symbol(text) => style(Style::default(), text),
        Token::Qualifier(text) => style(Color::Blue.into(), text),
        Token::Kind(text) => style(Color::Blue.into(), text),
        Token::Whitespace => style(Style::default(), " "),
        Token::Identifier(text) => style(Color::Cyan.into(), text),
        Token::Annotation(text) => style(Style::default(), text),
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
fn color_item_with_diff(diff_slice: &[diff::Result<&&Token>], is_old_item: bool) -> String {
    let styled_strings = diff_slice
        .iter()
        .filter_map(|diff_result| match diff_result {
            diff::Result::Left(&token) => is_old_item.then(|| {
                Color::Fixed(9)
                    .on(Color::Fixed(52))
                    .bold()
                    .paint(token.text())
            }),
            diff::Result::Both(&token, _) => Some(color_item_token(token, None)),
            diff::Result::Right(&token) => (!is_old_item).then(|| {
                Color::Fixed(10)
                    .on(Color::Fixed(22))
                    .bold()
                    .paint(token.text())
            }),
        })
        .collect::<Vec<_>>();

    AnsiStrings(&styled_strings).to_string()
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
