use crate::{OrdBySet, Order};

/// A drop guard that ensures the [`OrdBySet`] is properly sorted after any modifications
/// to the underlying reference are made
pub struct MutRefGuard<'set, T, Orderer: Order<T>>(
    pub(crate) &'set mut OrdBySet<T, Orderer>,
    pub(crate) usize,
);

impl<'set, T, Orderer: Order<T>> core::ops::Deref for MutRefGuard<'set, T, Orderer> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0.storage[self.1]
    }
}

impl<'set, T, Orderer: Order<T>> core::ops::DerefMut for MutRefGuard<'set, T, Orderer> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0.storage[self.1]
    }
}

impl<'set, T, Orderer: Order<T>> Drop for MutRefGuard<'set, T, Orderer> {
    fn drop(&mut self) {
        self.0.orderer.sort_slice(&mut self.0.storage);
    }
}
