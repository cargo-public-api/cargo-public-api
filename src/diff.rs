//! Contains facilities that allows you diff public APIs between releases and
//! commits. [`cargo
//! public-items`](https://github.com/Enselic/cargo-public-items) contains
//! additional helpers for that.

use crate::{
    tokens::{ChangedToken, ChangedTokenStream, Token, TokenStream},
    PublicItem,
};

/// An item has changed in the public API. Two [`PublicItem`]s are considered
/// the same if their `path` is the same.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ChangedPublicItem {
    /// How the item used to look.
    pub old: PublicItem,

    /// How the item looks now.
    pub new: PublicItem,
}

impl ChangedPublicItem {
    pub fn changed_tokens(&self) -> Vec<ChangedTokenStream> {
        ChangedTokenStream::new(ChangedPublicItem::align_tokens(
            &self.old.tokens,
            &self.new.tokens,
        ))
    }

    /// Calculates the difference between two TokenStreams, the algorithm is the Needleman-Wunsch algorithm
    fn align_tokens(a: &TokenStream, b: &TokenStream) -> Vec<ChangedToken> {
        // Reused code from an older project of mine, please go hard with the code refactoring suggestions ;-)
        // Match between sequences A and B.
        // First create a matrix of size [A, B].
        // With A as reference an Removed is something missing in A but present in B.
        // With A as reference a Inserted is something present in A but missing in B.

        let a: Vec<&Token> = a.tokens().collect();
        let b: Vec<&Token> = b.tokens().collect();
        #[derive(Clone)]
        enum Direction {
            Match,
            Removed,
            Inserted,
        }

        let mut matrix: Vec<Vec<(isize, Direction)>> =
            vec![vec![(0, Direction::Match); a.len() + 1]; b.len() + 1];

        fn max(
            (a1, a2): (isize, Direction),
            (b1, b2): (isize, Direction),
            (c1, c2): (isize, Direction),
        ) -> (isize, Direction) {
            if a1 >= b1 && a1 >= c1 {
                return (a1, a2);
            };
            if b1 >= a1 && b1 >= c1 {
                return (b1, b2);
            };
            if c1 >= a1 && c1 >= b1 {
                return (c1, c2);
            };
            unreachable!();
        }

        //Build the matrix
        for x in 0..a.len() {
            matrix[0][x] = (-(x as isize), Direction::Removed);
        }

        for x in 0..b.len() {
            matrix[x][0] = (-(x as isize), Direction::Inserted);
        }

        for x in 1..a.len() {
            for y in 1..b.len() {
                matrix[y][x] = max(
                    (matrix[y - 1][x].0 - 1, Direction::Inserted),
                    (matrix[y][x - 1].0 - 1, Direction::Removed),
                    (
                        matrix[y - 1][x - 1].0 + Token::align_score(a[x - 1], b[y - 1]),
                        Direction::Match,
                    ),
                );
            }
        }

        //Walk the matrix back
        let mut x: usize = a.len();
        let mut y: usize = b.len();
        let mut diffs = Vec::new();
        let mut diffs_return: Vec<ChangedToken>;
        loop {
            if x == 0 {
                diffs_return = b[0..y]
                    .iter()
                    .map(|i| ChangedToken::Removed((*i).clone()))
                    .collect();
                diffs.reverse();
                diffs_return.extend(diffs);
                break;
            }
            if y == 0 {
                diffs_return = a[0..x]
                    .iter()
                    .map(|i| ChangedToken::Inserted((*i).clone()))
                    .collect();
                diffs.reverse();
                diffs_return.extend(diffs);
                break;
            }
            match matrix[y][x].1 {
                Direction::Match => {
                    x -= 1;
                    y -= 1;
                    if Token::align_score(a[x], b[y]) < 0 {
                        diffs.push(ChangedToken::Inserted(b[y].clone()));
                        diffs.push(ChangedToken::Removed(a[x].clone()));
                    } else {
                        diffs.push(ChangedToken::Same(a[x].clone()));
                    }
                    continue;
                }
                Direction::Removed => {
                    x -= 1;
                    diffs.push(ChangedToken::Removed(a[x].clone()));
                    continue;
                }
                Direction::Inserted => {
                    y -= 1;
                    diffs.push(ChangedToken::Inserted(b[y].clone()));
                    continue;
                }
            }
        }

        diffs_return
    }
}

/// The return value of [`Self::between`]. To quickly get a sense of what it
/// contains, you can pretty-print it:
/// ```txt
/// println!("{:#?}", public_items_diff);
/// ```
#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublicItemsDiff {
    /// Items that have been removed from the public API. A MAJOR change, in
    /// semver terminology. Sorted.
    pub removed: Vec<PublicItem>,

    /// Items in the public API that has been changed. Generally a MAJOR change,
    /// but exceptions exist. For example, if the return value of a method is
    /// changed from `ExplicitType` to `Self` and `Self` is the same as
    /// `ExplicitType`.
    pub changed: Vec<ChangedPublicItem>,

    /// Items that have been added to public API. A MINOR change, in semver
    /// terminology. Sorted.
    pub added: Vec<PublicItem>,
}

impl PublicItemsDiff {
    /// Allows you to diff the public API between two arbitrary versions of a
    /// library, e.g. different releases. The input parameters `old` and `new`
    /// is the output of two different invocations of
    /// [`crate::public_items_from_rustdoc_json_str`].
    #[must_use]
    pub fn between(old_items: Vec<PublicItem>, new_items: Vec<PublicItem>) -> Self {
        let mut old_sorted = old_items;
        old_sorted.sort();

        let mut new_sorted = new_items;
        new_sorted.sort();

        // We can't implement this with sets, because different items might have
        // the same representations (e.g. because of limitations or bugs), so if
        // we used a Set, we would lose one of them.
        //
        // Our strategy is to only move items around, to reduce the risk of
        // duplicates and lost items.
        let mut removed: Vec<PublicItem> = vec![];
        let mut changed: Vec<ChangedPublicItem> = vec![];
        let mut added: Vec<PublicItem> = vec![];
        loop {
            match (old_sorted.pop(), new_sorted.pop()) {
                (None, None) => break,
                (Some(old), None) => {
                    removed.push(old);
                }
                (None, Some(new)) => {
                    added.push(new);
                }
                (Some(old), Some(new)) => {
                    if old != new && old.path == new.path {
                        // The same item, but there has been a change in type
                        changed.push(ChangedPublicItem { old, new });
                    } else {
                        match old.cmp(&new) {
                            std::cmp::Ordering::Less => {
                                added.push(new);

                                // Add it back and compare it again next
                                // iteration
                                old_sorted.push(old);
                            }
                            std::cmp::Ordering::Equal => {
                                // This is the same item, so just continue to
                                // the next pair
                                continue;
                            }
                            std::cmp::Ordering::Greater => {
                                removed.push(old);

                                // Add it back and compare it again next
                                // iteration
                                new_sorted.push(new);
                            }
                        }
                    }
                }
            }
        }

        // Make output predictable and stable
        removed.sort();
        changed.sort();
        added.sort();

        Self {
            removed,
            changed,
            added,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_and_only_item_removed() {
        let old = vec![item_with_path("foo")];
        let new = vec![];

        let actual = PublicItemsDiff::between(old, new);
        let expected = PublicItemsDiff {
            removed: vec![item_with_path("foo")],
            changed: vec![],
            added: vec![],
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn single_and_only_item_added() {
        let old = vec![];
        let new = vec![item_with_path("foo")];

        let actual = PublicItemsDiff::between(old, new);
        let expected = PublicItemsDiff {
            removed: vec![],
            changed: vec![],
            added: vec![item_with_path("foo")],
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn middle_item_added() {
        let old = vec![item_with_path("1"), item_with_path("3")];
        let new = vec![
            item_with_path("1"),
            item_with_path("2"),
            item_with_path("3"),
        ];

        let actual = PublicItemsDiff::between(old, new);
        let expected = PublicItemsDiff {
            removed: vec![],
            changed: vec![],
            added: vec![item_with_path("2")],
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn middle_item_removed() {
        let old = vec![
            item_with_path("1"),
            item_with_path("2"),
            item_with_path("3"),
        ];
        let new = vec![item_with_path("1"), item_with_path("3")];

        let actual = PublicItemsDiff::between(old, new);
        let expected = PublicItemsDiff {
            removed: vec![item_with_path("2")],
            changed: vec![],
            added: vec![],
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn aligned() {
        let a = vec![
            Token::identifier("a"),
            Token::Whitespace,
            Token::identifier("a"),
        ]
        .into();
        let b = vec![Token::identifier("a"), Token::identifier("a")].into();
        let aligned = ChangedPublicItem::align_tokens(&a, &b);
        assert_eq!(
            aligned,
            vec![
                ChangedToken::Same(Token::identifier("a")),
                ChangedToken::Removed(Token::Whitespace),
                ChangedToken::Same(Token::identifier("a"))
            ]
        )
    }

    #[test]
    fn aligned_fn_id_equality() {
        let a = vec![Token::identifier("a"), Token::identifier("a")].into();
        let b = vec![Token::function("a"), Token::identifier("a")].into();
        let aligned = ChangedPublicItem::align_tokens(&a, &b);
        assert_eq!(
            aligned,
            vec![
                ChangedToken::Same(Token::identifier("a")),
                ChangedToken::Same(Token::identifier("a"))
            ]
        )
    }

    fn item_with_path(path: &str) -> PublicItem {
        PublicItem {
            path: path.split("::").map(|i| i.to_string()).collect(),
            tokens: TokenStream::default(),
        }
    }
}
