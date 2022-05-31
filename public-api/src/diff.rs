//! Contains facilities that allows you diff public APIs between releases and
//! commits. [`cargo
//! public-api`](https://github.com/Enselic/cargo-public-api) contains
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
/// println!("{:#?}", public_api_diff);
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
    /// [`crate::public_api_from_rustdoc_json_str`].
    #[must_use]
    pub fn between(old_items: Vec<PublicItem>, new_items: Vec<PublicItem>) -> Self {
        let mut old_sorted = old_items;
        old_sorted.sort();

        let mut new_sorted = new_items;
        new_sorted.sort();

        (old_sorted, new_sorted) = Self::remove_pure_duplicates(old_sorted, new_sorted);

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

    /// Removes all pairs of exact duplicates. This prevents a "off-by-one" diff
    /// error where a series of changed items is shown, when they are in fact
    /// equal, but just not detected as such.
    fn remove_pure_duplicates(
        mut old_sorted: Vec<PublicItem>,
        mut new_sorted: Vec<PublicItem>,
    ) -> (Vec<PublicItem>, Vec<PublicItem>) {
        let mut old_reduced = Vec::with_capacity(old_sorted.len());
        let mut new_reduced = Vec::with_capacity(new_sorted.len());

        loop {
            match (old_sorted.pop(), new_sorted.pop()) {
                // There are no items left to process. This means we are done.
                (None, None) => break,

                // If there is only old items or only new items left, they
                // should all be part of the upcoming "real" diff. So just add
                // them to the result
                (Some(old), None) => {
                    old_reduced.push(old);
                }
                (None, Some(new)) => {
                    new_reduced.push(new);
                }

                (Some(old), Some(new)) => {
                    match old.cmp(&new) {
                        std::cmp::Ordering::Less => {
                            // The last new item is "further down" than last old
                            // item. This means there can be no new item that is
                            // exactly equal to any equal to this old item. So
                            // move the new item to the next stage.
                            new_reduced.push(new);

                            // There will now be a new new item to compare
                            // against. It might be equal to ths current old
                            // item. So add the old item back so we can compare
                            // again with the next new item.
                            old_sorted.push(old);
                        }
                        std::cmp::Ordering::Equal => {
                            // The items are exactly the same. There is no need
                            // to include them in the next diff step. So don't
                            // add them to any vec.
                        }
                        std::cmp::Ordering::Greater => {
                            // Same as Ordering::Less above but reversed.
                            old_reduced.push(old);
                            new_sorted.push(new);
                        }
                    }
                }
            }
        }

        // The order has now been reversed from before. Reverse back. We could
        // probably do it with .reverse(). But be extra safe and do it with
        // .sort().
        old_reduced.sort();
        new_reduced.sort();

        (old_reduced, new_reduced)
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
            tokens: vec![crate::tokens::Token::identifier(path)],
        }
    }
}
