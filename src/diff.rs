//! Contains facilities that allows you diff public APIs between releases and
//! commits. [`cargo
//! public-items`](https://github.com/Enselic/cargo-public-items) contains
//! additional helpers for that.

use crate::PublicItem;

/// An item has changed in the public API. Two [`PublicItem`]s are considered
/// the same if their `path` is the same.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ChangedPublicItem {
    /// How the item used to look.
    pub old: PublicItem,

    /// How the item looks now.
    pub new: PublicItem,
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

    fn item_with_path(path: &str) -> PublicItem {
        PublicItem {
            path: path
                .split("::")
                .map(std::string::ToString::to_string)
                .collect(),
            tokens: crate::tokens::TokenStream::default(),
        }
    }
}
