//! A library providing a weakly ordered multi-set with compile-time configurable
//! ordering scheme.
//!
//! ### When To Use This
//!
//! * When you want a [`BTreeSet`](alloc::collections::BTreeSet) but your data involves
//! partial/loose equivelance, and you want to be able to perform efficient retrievals of
//! multiple values of loose equivelance.
//! * When you have ordered keys stored in the same type as the values, allowing
//! a [`BTreeMap`](alloc::collections::BTreeMap)-like data structure but with inline
//! keys.
//!     * This is done by using a custom [`Order`] implementation in order to order
//!     types by the fields being used as keys, without a reliance on being totally ordered
//! * When you want a multi-{set, map} but hashing is not an option
//!
//! ### When Not To Use This
//!
//! * In place of `HashMap`/`HashSet`/`BTreeMap`/`BTreeSet` when you don't need multiple
//! loosely equivelant values.
//!
//!
//! ## Overview
//!
//! An [`OrdBySet`] is composed of two parts: its storage backing (a sorted `Vec<T>`)
//! and a user-provided orderer. An orderer is a value which can take two items and
//! loosely compare them. This is done via the [`Order<T>`] trait, which requires a
//! single method, [`order_of`](Order::order_of):
//!
//! ```
//! # use std::cmp::Ordering;
//! # trait Order<T> {
//! fn order_of(&self, left: &T, right: &T) -> Ordering;
//! # }
//! ```
//!
//! Unlike [`Ord`], however, this is not guaranteed to be [totally ordered], and as
//! such it can be used in such a manner that groups loosely-equivelant values, similarly
//! to how a [Bag datastructure] allows for storing multiple of the same value.
//!
//! [totally ordered]: https://wikipedia.org/wiki/Total_order
//! [Bag datastructure]: https://docs.rs/hashbag/latest/hashbag/struct.HashBag.html
//!
//! The differentiating feature, however, is that one can then proceed to query all
//! losely equivelant types[^1]. The ordering scheme.
//!
//! [^1]: One example being that you might want a query of 3 to turn up both 3 as an
//! integer and 3 as a string, while still storing both the string and the integer.
//! For more info on this see [`Order`]'s docs.
//!
//! ### Example
//!
//! ```
//! use ord_by_set::OrdBySet;
//!
//! // Our orderer will be a simple function that sorts based on the first 5 characters
//! let ordering_fn = |left: &&str, right: &&str| left[..5].cmp(&right[..5]);
//!
//! let set = OrdBySet::new_with_order(ordering_fn)
//!     .with_items(["00001_foo", "00001_bar", "00002_foo"]);
//!
//! let id_1_subset = set.get(&"00001").unwrap();
//!
//! // id_1_subset = unordered(["00001_foo", "00001_bar"])
//! assert_eq!(id_1_subset.len(), 2);
//! assert!(id_1_subset.contains(&"00001_bar"));
//! assert!(id_1_subset.contains(&"00001_foo"));
//! ```
//!
//! While the above uses a closure for the orderer, it can be any type if you implement
//! [`Order<T>`]. Typically this is done via a [zero-sized type] as usually state is not
//! needed by the ordering mechanism, just behavior:
//!
//! ```
//! # use ord_by_set::{OrdBySet, Order};
//! # use std::cmp::Ordering;
//! #[derive(Default)]
//! struct EverythingEqual;
//!
//! impl<T> Order<T> for EverythingEqual {
//!     fn order_of(&self, _: &T, _: &T) -> Ordering {
//!         Ordering::Equal
//!     }
//! }
//!
//! type AllEqualSet = OrdBySet<i32, EverythingEqual>;
//!
//! let mut set = AllEqualSet::new().with_items([3, 5, 2, 7]);
//!
//! assert_eq!(set.count(&30), 4);
//! set.remove_all(&0);
//! assert!(set.is_empty());
//! ```
//!
//! [zero-sized type]: https://doc.rust-lang.org/nomicon/exotic-sizes.html#zero-sized-types-zsts
#![no_std]
use core::ops::Range;

extern crate alloc;
use alloc::vec::Vec;

mod mut_ref_guard;
mod order;
mod slice_guard;
mod trait_impls;

pub use {
    mut_ref_guard::MutRefGuard,
    order::{FullOrd, Order},
    slice_guard::SliceGuard,
};

#[cfg(test)]
mod tests;

/// A multi-set backed by a sorted list of items while allowing for a custom
/// ordering scheme.
#[derive(Clone, Hash)]
pub struct OrdBySet<T, Orderer = FullOrd>
where
    Orderer: Order<T>,
{
    storage: Vec<T>,
    orderer: Orderer,
}

impl<T, Orderer: Order<T> + Default> OrdBySet<T, Orderer> {
    /// Create an empty `OrdBySet` with a default-initialized orderer
    pub fn new() -> Self {
        Self::default()
    }
}

impl<T: Ord> OrdBySet<T, FullOrd> {
    /// Create an empty `OrdBySet` where the set is fully ordered using [`Ord`]
    pub fn fully_ordered() -> Self {
        Self::new()
    }
}

impl<T, Orderer: Order<T>> OrdBySet<T, Orderer> {
    /// Create an empty `OrdBySet` with a custom ordering scheme
    pub fn new_with_order(orderer: Orderer) -> Self {
        Self {
            storage: Vec::new(),
            orderer,
        }
    }

    /// Inserts an item into the set. This operation is more efficient when items are
    /// inserted in-order due to being backed by contiguous memory (a `Vec`), and thus
    /// shares a lot of the same performance properties of `Vec`.
    ///
    /// ### Example
    ///
    /// ```
    /// use ord_by_set::OrdBySet;
    ///
    /// let mut set = OrdBySet::fully_ordered();
    /// set.insert(0);
    /// set.insert(1);
    /// set.insert(70);
    /// set.insert(1);
    ///
    /// assert_eq!(set.len(), 4);
    /// assert_eq!(set.count(&1), 2);
    /// ```
    pub fn insert(&mut self, item: T) {
        let insertion_point = self
            .storage
            .binary_search_by(|x| self.orderer.order_of(&x, &item))
            .unwrap_or_else(|insert_at| insert_at);

        self.storage.insert(insertion_point, item);
    }

    fn get_index_range_of(&self, item: &T) -> Option<Range<usize>> {
        let start = self
            .storage
            .partition_point(|probe| self.orderer.order_of(&probe, &item).is_lt());
        let len = self.storage[start..]
            .partition_point(|probe| self.orderer.order_of(&probe, &item).is_eq());
        let end = start + len;

        (end > start).then(|| start..end)
    }

    /// Removes all values from the set where the orderer determines the value is
    /// equal to the provided item. Returns `true` if any items were removed.
    pub fn remove_all(&mut self, item: &T) -> bool {
        if let Some(range) = self.get_index_range_of(item) {
            // drop to ensure elements are removed immediately.
            drop(self.storage.drain(range));

            true
        } else {
            false
        }
    }

    /// Removes the first value from the set where the orderer determines the value is
    /// equal to the provided item. Returns the item if it is removed.
    pub fn remove_first(&mut self, item: &T) -> Option<T> {
        let location_range = self.get_index_range_of(item)?;
        let contains_item = !location_range.is_empty();

        contains_item.then(|| self.storage.remove(location_range.start))
    }

    /// Removes all equivelant values from the set, returning all the items which
    /// were found to be equal and removed.
    pub fn drain(&mut self, item: &T) -> Vec<T> {
        self.get_index_range_of(item)
            .map(|range| self.storage.drain(range).collect())
            .unwrap_or_default()
    }

    /// Retains only the elements specified by the predicate, removing all elements
    /// where the provided predicate returns `false`.
    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&T) -> bool,
    {
        self.storage.retain(f)
    }

    /// Get a slice of all equivelant items. No sorting order within is guaranteed.
    ///
    /// Returns `None` if no matching items were found in the set.
    pub fn get(&self, item: &T) -> Option<&[T]> {
        Some(&self.storage[self.get_index_range_of(item)?])
    }

    /// Get the first item in the set found while binary searching for a given equivelant
    /// no guarantee is found that the item is the first in contiguous memory, rather,
    /// this finds the quickest item to be found.
    pub fn get_first(&self, item: &T) -> Option<&T> {
        let index = self
            .storage
            .binary_search_by(|x| self.orderer.order_of(&x, item))
            .ok()?;

        self.storage.get(index)
    }

    /// Get a slice of all equivelant items. No sorting order within is guaranteed
    ///
    /// **Note:** the state of the `OrdBySet` is unspecified if this [`SliceGuard`] is
    /// not dropped, such as via `mem::forget`.
    pub fn get_mut(&mut self, item: &T) -> Option<SliceGuard<'_, T, Orderer>> {
        let range = self.get_index_range_of(item)?;

        Some(SliceGuard(self, range))
    }

    /// Get a mutable reference to the first item in the set found while binary searching
    /// for a given equivelant no guarantee is found that the item is the first in
    /// contiguous memory, rather, this finds the quickest item to be found.
    ///
    /// **Note:** the state of the `OrdBySet` is unspecified if this [`MutRefGuard`] is
    /// not dropped, such as via `mem::forget`.
    pub fn get_first_mut(&mut self, item: &T) -> Option<MutRefGuard<'_, T, Orderer>> {
        let index = self
            .storage
            .binary_search_by(|x| self.orderer.order_of(&x, item))
            .ok()?;

        Some(MutRefGuard(self, index))
    }

    /// Check if an equivelant item is contained in the set
    pub fn contains(&self, item: &T) -> bool {
        self.storage
            .binary_search_by(|x| self.orderer.order_of(&x, item))
            .is_ok()
    }

    /// Check the number of equivelant items contained in the set
    pub fn count(&self, item: &T) -> usize {
        self.get_index_range_of(item)
            .map(|range| range.len())
            .unwrap_or(0)
    }

    /// Returns an iterator over all of the elements in no specified order
    pub fn iter(&self) -> impl Iterator<Item = &T> + '_ {
        self.storage.iter()
    }

    /// Returns an iterator over all of the elements in no specified order such that
    /// each value can be modified.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> + '_ {
        self.storage.iter_mut()
    }

    /// Replaces the contents of the set with the contents of a `Vec`
    ///
    /// ## Example
    ///
    /// ```
    /// use ord_by_set::OrdBySet;
    ///
    /// let set: OrdBySet<u64> = OrdBySet::new().with_items([3, 1, 3, 2]);
    /// assert_eq!(set.count(&3), 2);
    /// ```
    pub fn with_items<Items: Into<Vec<T>>>(self, items: Items) -> Self {
        let mut storage = items.into();
        self.orderer.sort_slice(&mut storage);

        Self { storage, ..self }
    }

    /// Get the number of items in the set
    pub fn len(&self) -> usize {
        self.storage.len()
    }

    /// Returns the number of elements the set can hold without reallocating
    pub fn capacity(&self) -> usize {
        self.storage.capacity()
    }

    /// Remove all items in the set
    pub fn clear(&mut self) {
        self.storage.truncate(0);
    }

    /// Checks if there are any items inside the set
    pub fn is_empty(&self) -> bool {
        self.storage.is_empty()
    }

    fn range_to_index_range(&self, low: &T, high: &T) -> Option<Range<usize>> {
        if !self.orderer.order_of(low, high).is_lt() {
            return None;
        }

        let start = self
            .storage
            .partition_point(|probe| self.orderer.order_of(probe, low).is_lt());

        let len = self.storage[start..]
            .partition_point(|probe| self.orderer.order_of(probe, high).is_le());

        let end = start + len;

        (end > start).then(|| start..end)
    }

    /// Gets a slice of all elements inclusively between two bounds
    pub fn range(&self, low: &T, high: &T) -> Option<&[T]> {
        self.range_to_index_range(low, high)
            .map(|range| &self.storage[range])
    }

    /// Gets a mutable slice of all elements between two bounds
    pub fn range_mut(&mut self, low: &T, high: &T) -> Option<SliceGuard<'_, T, Orderer>> {
        self.range_to_index_range(low, high)
            .map(move |range| SliceGuard(self, range))
    }
}

impl<T, Orderer: Order<T>> OrdBySet<T, Orderer>
where
    T: PartialEq,
{
    /// Searches for a specific item (based on `PartialEq`) and removes it, returning it
    /// if it exists.
    ///
    /// If multiple exist, the first found is removed.
    ///
    /// **Note:** this method assumes that the equality of `Orderer` is a superset of
    /// `PartialEq`. That is to say that if `x == y` (`PartialEq`), then
    /// `orderer.order_of(&x, &y)` must return `Ordering::Equal`.
    pub fn remove_specific(&mut self, val: &T) -> Option<T> {
        let location_range = self.get_index_range_of(val)?;
        let start = location_range.start;
        let index = self.storage[location_range].iter().position(|x| x == val)? + start;

        Some(self.storage.remove(index))
    }

    /// Searches for a specific item (based on `PartialEq`) and returns a reference to it.
    ///
    /// If multiple exist, the first found is returned.
    ///
    /// **Note:** this method assumes that the equality of `Orderer` is a superset of
    /// `PartialEq`. That is to say that if `x == y` (`PartialEq`), then
    /// `orderer.order_of(&x, &y)` must return `Ordering::Equal`.
    pub fn get_specific(&self, val: &T) -> Option<&T> {
        let location_range = self.get_index_range_of(val)?;
        let start = location_range.start;
        let index = self.storage[location_range].iter().position(|x| x == val)? + start;

        self.storage.get(index)
    }

    /// Searches for a specific item (based on [`PartialEq`]) and returns a mutable
    /// reference to the value.
    ///
    /// If multiple exist, the first found is returned.
    ///
    /// **Note:** this method assumes that the equality of `Orderer` is a superset of
    /// `PartialEq`. That is to say that if `x == y` (`PartialEq`), then
    /// `orderer.order_of(&x, &y)` must return `Ordering::Equal`.
    pub fn get_specific_mut(&mut self, val: &T) -> Option<MutRefGuard<'_, T, Orderer>> {
        let location_range = self.get_index_range_of(val)?;
        let start = location_range.start;
        let index = self.storage[location_range].iter().position(|x| x == val)? + start;

        Some(MutRefGuard(self, index))
    }

    /// Returns `true` if a specific item (based on [`PartialEq`]) exists in the set.
    ///
    /// **Note:** this method assumes that the equality of `Orderer` is a superset of
    /// `PartialEq`. That is to say that if `x == y` (`PartialEq`), then
    /// `orderer.order_of(&x, &y)` must return `Ordering::Equal`.
    pub fn contains_specific(&self, val: &T) -> bool {
        if let Some(location_range) = self.get_index_range_of(val) {
            self.storage[location_range].iter().any(|x| x == val)
        } else {
            false
        }
    }
}
