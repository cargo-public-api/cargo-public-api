use std::io::{Result, Write};

use ansi_term::{ANSIString, ANSIStrings, Color, Style};
use itertools::Itertools;
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
                    let diff_chars =
                        diff::chars(&changed_item.old.to_string(), &changed_item.new.to_string());

                    writeln!(
                        w,
                        "-{}\n+{}",
                        color_item_with_diff(&changed_item.old, &diff_chars, true),
                        color_item_with_diff(&changed_item.new, &diff_chars, false),
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

fn color_item_with_diff(
    item: &PublicItem,
    diff_chars: &[diff::Result<char>],
    is_old_item: bool,
) -> String {
    let diff_style = if is_old_item {
        Color::Fixed(9).on(Color::Fixed(52)).bold()
    } else {
        Color::Fixed(10).on(Color::Fixed(22)).bold()
    };

    // Create a series of batches of `Some(style)` or `None` depending on whether each given char
    // should be styled as a diff.
    let mut diff_sequences = vec![];
    diff_chars.iter().for_each(|result| match result {
        diff::Result::Left(_) => {
            if is_old_item {
                diff_sequences.push(Some(diff_style));
            }
        }
        diff::Result::Both(..) => {
            diff_sequences.push(None);
        }
        diff::Result::Right(_) => {
            if !is_old_item {
                diff_sequences.push(Some(diff_style));
            }
        }
    });

    // Create the default coloured strings for this collection of tokens.
    let default_coloured_strings = item
        .tokens()
        .map(|t| color_item_token(t, None))
        .collect::<Vec<_>>();
    let ansi_strings = ANSIStrings(&default_coloured_strings);

    // Collect the modified and unmodified substrings.
    let mut diff_strings = vec![];
    let mut seq_start_index = 0;
    // Turn the batches of optional styles into an iterator of tuples of (num of consecutive
    // identical, optional style).
    for (len, maybe_style) in diff_sequences.into_iter().dedup_with_count() {
        let mut sub_strings = ansi_term::sub_string(seq_start_index, len, &ansi_strings);
        // If this batch of chars should have a diff style, apply it to the sub-strings.
        if let Some(style) = maybe_style {
            for sub_string in &mut sub_strings {
                *sub_string.style_ref_mut() = style;
            }
        }
        diff_strings.extend(sub_strings);
        seq_start_index += len;
    }

    ANSIStrings(&diff_strings).to_string()
}
