#![no_std]
use core::cmp::Ordering;
use core::ops::Range;

extern crate alloc;
use alloc::vec::Vec;

/// A multi-set backed by a sorted list of items while allowing for a custom
/// ordering scheme.
pub struct OrdBySet<T, Orderer = FullOrd>
where
    Orderer: Order<T>,
{
    storage: Vec<T>,
    orderer: Orderer,
}

impl<T, Orderer: Order<T> + Default> Default for OrdBySet<T, Orderer> {
    fn default() -> Self {
        Self {
            storage: Vec::default(),
            orderer: Orderer::default(),
        }
    }
}

impl<T, Orderer: Order<T> + Default> OrdBySet<T, Orderer> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<T, Orderer: Order<T>> OrdBySet<T, Orderer> {
    pub fn new_with_order(orderer: Orderer) -> Self {
        Self {
            storage: Vec::new(),
            orderer,
        }
    }

    /// Inserts an item into the set. This operation is more efficient when items are
    /// inserted in-order due to being backed by contiguous memory (a `Vec`), and thus
    /// shares a lot of the same performance properties of `Vec`.
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

    /// Removes all equivelant values from the set, returning all the items which
    /// were found to be equal and removed.
    pub fn drain(&mut self, item: &T) -> Vec<T> {
        self.get_index_range_of(item)
            .map(|range| self.storage.drain(range).collect())
            .unwrap_or_default()
    }

    /// Get a slice of all equivelant items. No sorting order within is guaranteed.
    ///
    /// Returns `None` if no matching items were found in the set.
    pub fn get<'a>(&'a self, item: &T) -> Option<&'a [T]> {
        Some(&self.storage[self.get_index_range_of(item)?])
    }

    /// Get the first item in the set found while binary searching for a given equivelant
    /// no guarantee is found that the item is the first in contiguous memory, rather,
    /// this finds the quickest item to be found.
    pub fn get_first<'a>(&'a self, item: &T) -> Option<&'a T> {
        let index = self
            .storage
            .binary_search_by(|x| self.orderer.order_of(&x, item))
            .ok()?;

        Some(&self.storage[index])
    }

    /// Get a slice of all equivelant items. No sorting order within is guaranteed
    pub fn get_mut<'a>(&'a mut self, item: &T) -> Option<&'a mut [T]> {
        let range = self.get_index_range_of(item)?;

        Some(&mut self.storage[range])
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
}

/// An ordering implementation that just defers to [`Ord`]
#[derive(Default)]
pub struct FullOrd;

pub trait Order<T> {
    fn order_of(&self, left: &T, right: &T) -> Ordering;
}

impl<T: Ord> Order<T> for FullOrd {
    fn order_of(&self, left: &T, right: &T) -> Ordering {
        left.cmp(right)
    }
}

#[cfg(test)]
mod tests;
