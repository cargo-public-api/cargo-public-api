//! This entire module is a vendoring/backport of [Implement
//! `HashBag::difference(&self, other:
//! &HashBag`](https://github.com/jonhoo/hashbag/pull/6)
//!
//! Once/if that PR is merged and made available through a new release of
//! [`hashbag`](https://crates.io/crates/hashbag) we can remove all of this
//! code.

use hashbag::{HashBag, Iter};
use std::{
    collections::{hash_map::RandomState, HashMap},
    hash::{BuildHasher, Hash},
};

pub trait DifferenceAddon<'a, T, S>
where
    T: Eq + Hash,
    S: BuildHasher,
{
    fn difference(&'a self, other: &'a HashBag<T, S>) -> Difference<'a, T, S>;
}

impl<'a, T, S> DifferenceAddon<'a, T, S> for HashBag<T, S>
where
    T: Eq + Hash,
    S: BuildHasher,
{
    fn difference(&'a self, other: &'a HashBag<T, S>) -> Difference<'a, T, S> {
        Difference {
            base_iter: self.iter(),
            other,
            removed_from_other: HashMap::new(),
        }
    }
}

/// This `struct` is created by [`HashBag::difference`].
/// See its documentation for more info.
pub struct Difference<'a, T, S = RandomState> {
    /// An iterator over "self"
    base_iter: Iter<'a, T>,

    /// The bag with entries we DO NOT want to return
    other: &'a HashBag<T, S>,

    /// Keeps track of many times we have conceptually "consumed" an entry from
    /// `other`.
    removed_from_other: HashMap<&'a T, usize>,
}

impl<'a, T, S> Iterator for Difference<'a, T, S>
where
    T: Eq + Hash,
    S: BuildHasher,
{
    type Item = &'a T;

    #[inline]
    fn next(&mut self) -> Option<&'a T> {
        loop {
            let next = self.base_iter.next()?;
            let removal_count = self.removed_from_other.entry(next).or_insert(0);

            // Keep track of how many times we have removed the current entry.
            // We don't actually remove anything, we just pretend we do.
            *removal_count += 1;

            // If we removed MORE entries from `other`, THEN we may start
            // returning entries from the base iterator.
            if *removal_count > self.other.contains(next) {
                return Some(next);
            }
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (_, upper_bound) = self.base_iter.size_hint();
        (0, upper_bound)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_difference_from_empty() {
        do_test_difference(&[], &[], &[]);
        do_test_difference(&[], &[1], &[]);
        do_test_difference(&[], &[1, 1], &[]);
        do_test_difference(&[], &[1, 1, 2], &[]);
    }

    #[test]
    fn test_difference_from_one() {
        do_test_difference(&[1], &[], &[1]);
        do_test_difference(&[1], &[1], &[]);
        do_test_difference(&[1], &[1, 1], &[]);
        do_test_difference(&[1], &[2], &[1]);
        do_test_difference(&[1], &[1, 2], &[]);
        do_test_difference(&[1], &[2, 2], &[1]);
    }

    #[test]
    fn test_difference_from_duplicate_ones() {
        do_test_difference(&[1, 1], &[], &[1, 1]);
        do_test_difference(&[1, 1], &[1], &[1]);
        do_test_difference(&[1, 1], &[1, 1], &[]);
        do_test_difference(&[1, 1], &[2], &[1, 1]);
        do_test_difference(&[1, 1], &[1, 2], &[1]);
        do_test_difference(&[1, 1], &[2, 2], &[1, 1]);
    }

    #[test]
    fn test_difference_from_one_one_two() {
        do_test_difference(&[1, 1, 2], &[], &[1, 1, 2]);
        do_test_difference(&[1, 1, 2], &[1], &[1, 2]);
        do_test_difference(&[1, 1, 2], &[1, 1], &[2]);
        do_test_difference(&[1, 1, 2], &[2], &[1, 1]);
        do_test_difference(&[1, 1, 2], &[1, 2], &[1]);
        do_test_difference(&[1, 1, 2], &[2, 2], &[1, 1]);
    }

    #[test]
    fn test_difference_from_larger_bags() {
        do_test_difference(&[1, 2, 2, 3], &[3], &[1, 2, 2]);
        do_test_difference(&[1, 2, 2, 3], &[4], &[1, 2, 2, 3]);
        do_test_difference(&[2, 2, 2, 2], &[2, 2], &[2, 2]);
        do_test_difference(&[2, 2, 2, 2], &[], &[2, 2, 2, 2]);
    }

    fn do_test_difference(
        self_entries: &[isize],
        other_entries: &[isize],
        expected_entries: &[isize],
    ) {
        let this = self_entries.iter().collect::<HashBag<_>>();
        let other = other_entries.iter().collect::<HashBag<_>>();
        let expected = expected_entries.iter().collect::<HashBag<_>>();
        assert_eq!(
            this.difference(&other)
                .copied()
                .collect::<HashBag<&isize>>(),
            expected
        );
    }
}
