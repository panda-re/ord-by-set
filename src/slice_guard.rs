use crate::{OrdBySet, Order};
use core::ops::Range;

/// A drop guard that ensures the [`OrdBySet`] is properly sorted after any modifications
/// to the underlying slice are made
pub struct SliceGuard<'set, T, Orderer: Order<T>>(
    pub(crate) &'set mut OrdBySet<T, Orderer>,
    pub(crate) Range<usize>,
);

impl<'set, T, Orderer: Order<T>> core::ops::Deref for SliceGuard<'set, T, Orderer> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.0.storage[self.1.clone()]
    }
}

impl<'set, T, Orderer: Order<T>> core::ops::DerefMut for SliceGuard<'set, T, Orderer> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0.storage[self.1.clone()]
    }
}

impl<'set, T, Orderer: Order<T>> Drop for SliceGuard<'set, T, Orderer> {
    fn drop(&mut self) {
        self.0.orderer.sort_slice(&mut self.0.storage);
    }
}
