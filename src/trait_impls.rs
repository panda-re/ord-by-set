use crate::{OrdBySet, Order};
use alloc::vec::Vec;
use core::fmt::Debug;

impl<T, Orderer: Order<T>> IntoIterator for OrdBySet<T, Orderer> {
    type IntoIter = alloc::vec::IntoIter<T>;
    type Item = T;

    fn into_iter(self) -> Self::IntoIter {
        self.storage.into_iter()
    }
}

impl<T, Orderer: Order<T> + Default> From<Vec<T>> for OrdBySet<T, Orderer> {
    fn from(mut storage: Vec<T>) -> Self {
        let orderer = Orderer::default();

        storage.sort_by(|left, right| orderer.order_of(&left, &right));

        Self { storage, orderer }
    }
}

impl<T, Orderer: Order<T> + Default> FromIterator<T> for OrdBySet<T, Orderer> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self::from(iter.into_iter().collect::<Vec<_>>())
    }
}

impl<T: Debug, Orderer: Order<T>> Debug for OrdBySet<T, Orderer> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.storage.fmt(f)
    }
}

impl<T, Orderer: Order<T> + Default> Default for OrdBySet<T, Orderer> {
    fn default() -> Self {
        Self {
            storage: Vec::default(),
            orderer: Orderer::default(),
        }
    }
}
