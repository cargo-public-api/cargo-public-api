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

/// Returns a styled string similar to `color_item_token`, but where whole tokens are highlighted if
/// they contain a difference.
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

    let mut diff_chars_iter = diff_chars.iter();
    let styled_strings = item
        .tokens()
        .map(|token| {
            if contains_diff(&mut diff_chars_iter, token, is_old_item) {
                diff_style.paint(token.text())
            } else {
                color_item_token(token, None)
            }
        })
        .collect::<Vec<_>>();

    ANSIStrings(&styled_strings).to_string()
}

/// Returns `true` if any of the iterator values indicates a diff is present.  Old items equate to
/// the left side of the diff, and new items to the right.
fn contains_diff<'a>(
    diff_chars_iter: &mut impl Iterator<Item = &'a diff::Result<char>>,
    token: &Token,
    is_old_item: bool,
) -> bool {
    let mut has_difference = false;
    // We need to ensure we consume all `token.len()` entries to have the mutable iterator aligned
    // to the start of the next token when finished here, e.g. we can't use `Iterator::any`.
    let diff_string: String = diff_chars_iter
        .filter_map(|diff_char| match diff_char {
            diff::Result::Left(char) => {
                if is_old_item {
                    has_difference = true;
                    Some(char)
                } else {
                    None
                }
            }
            diff::Result::Both(char, _) => Some(char),
            diff::Result::Right(char) => {
                if is_old_item {
                    None
                } else {
                    has_difference = true;
                    Some(char)
                }
            }
        })
        .take(token.len())
        .collect();
    debug_assert_eq!(token.text(), diff_string);
    // If `has_difference` is still `false`, we need to check that we didn't skip all iterations due
    // to having an empty token.
    has_difference || (diff_string.is_empty() && token.text().is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_identify_diff() {
        fn assert_different(different: &str) {
            let initial_string = String::from("first string");
            let old_token = Token::Identifier(initial_string.clone());
            let new_token = Token::Identifier(different.to_string());
            let diff_chars = diff::chars(&initial_string, different);
            assert!(
                contains_diff(&mut diff_chars.iter(), &old_token, true),
                "contains_diff should return true for '{}' vs '{}'",
                initial_string,
                different
            );
            assert!(
                contains_diff(&mut diff_chars.iter(), &new_token, false),
                "contains_diff should return true for '{}' vs '{}'",
                initial_string,
                different
            );
        }

        assert_different("girst string");
        assert_different("first strinh");
        assert_different("first-string");
        assert_different("");
        assert_different("aaa");
    }

    #[test]
    fn should_identify_no_diff() {
        let initial_string = String::from("first string");
        let token = Token::Identifier(initial_string.clone());
        let diff_chars = diff::chars(&initial_string, &initial_string);
        assert!(
            !contains_diff(&mut diff_chars.iter(), &token, true),
            "contains_diff should return false for '{}' vs '{}'",
            initial_string,
            initial_string,
        );
        assert!(
            !contains_diff(&mut diff_chars.iter(), &token, false),
            "contains_diff should return false for '{}' vs '{}'",
            initial_string,
            initial_string,
        );
    }
}
