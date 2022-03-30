//! Contains facilities that allows you diff public APIs between releases and
//! commits. [`cargo
//! public-items`](https://github.com/Enselic/cargo-public-items) contains
//! additional helpers for that.

use std::collections::HashSet;

use crate::PublicItem;

/// An item has changed in the public API. Two [`PublicItem`]s are considered
/// the same if their `path` is the same.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
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
#[derive(Debug)]
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
    pub fn between(old: Vec<PublicItem>, new: Vec<PublicItem>) -> Self {
        let old_set: HashSet<_> = HashSet::from_iter(old);
        let new_set: HashSet<_> = HashSet::from_iter(new);

        // Using a Set here relies on that two different items do not look the
        // same. This is currently not guaranteed due to
        // https://github.com/Enselic/public_items/issues/16, but this algorithm
        // will have to do for now. In real world use, it is not very common for
        // two different items to look the same.
        let mut added_set: HashSet<_> = new_set.difference(&old_set).cloned().collect();
        let mut removed_set: HashSet<_> = old_set.difference(&new_set).cloned().collect();
        let mut changed = vec![];

        // Find what items to move from `added` and `removed` to `changed`. We
        // use the strategy of moving to make sure that we never lose an item.
        // Even if the algorithm is buggy and does not find all items that
        // should be reported as changes, we can be confident that the items
        // will at least remain in `added` and `removed` and not get lost, which
        // is very important.
        let mut move_to_changed = vec![];
        for removed_item in &removed_set {
            if let Some(added_item) = added_set
                .iter()
                .find(|added_item| added_item.0.path == removed_item.0.path)
            {
                move_to_changed.push((removed_item.clone(), added_item.clone()));
            }
        }

        for pair in move_to_changed {
            changed.push(ChangedPublicItem {
                old: removed_set
                    .take(&pair.0)
                    .expect("it must exist because we used it above!"),
                new: added_set
                    .take(&pair.1)
                    .expect("it must exist because we found it above!"),
            });
        }

        let mut removed: Vec<_> = removed_set.into_iter().collect();
        removed.sort();

        changed.sort();

        let mut added: Vec<_> = added_set.into_iter().collect();
        added.sort();

        Self {
            removed,
            changed,
            added,
        }
    }
}
