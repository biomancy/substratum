use crate::memory::pool::lifecycle::{
    FromPool, IntoPool, PoolAcquireHandle, PoolReleaseHandle, TryFromPool,
};
use crate::memory::ArrayAllocation;
use std::any::TypeId;

impl<T: IntoPool> IntoPool for Vec<T> {
    type Identity = Vec<T::Identity>;
    fn into_pool(mut self, handle: &mut impl PoolReleaseHandle) {
        for item in self.drain(..) {
            handle.release(item);
        }

        if let Some(array) = ArrayAllocation::from_vec(self) {
            handle.put_array(TypeId::of::<Self::Identity>(), array)
        }
    }
}

impl<T: IntoPool> FromPool for Vec<T> {
    fn from_pool(handle: &mut impl PoolAcquireHandle) -> Self {
        Self::try_from_pool(handle).unwrap_or_default()
    }
}

impl<T: IntoPool> TryFromPool for Vec<T> {
    fn try_from_pool(handle: &mut impl PoolAcquireHandle) -> Option<Self> {
        let array = handle.take_array(
            TypeId::of::<Self::Identity>(),
            align_of::<T>(),
            size_of::<T>(),
        )?;
        let vec = array.into_vec().expect(
            "A failed conversion from `ArrayAllocation` to `Vec<T>` is a fatal error. \
            If this occurs, it indicates a bug in the employed memory pool implementation.",
        );
        Some(vec)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::memory::pool::{PoolAcquire, PoolLease, PoolRelease};
    use crate::memory::Pool;
    #[test]
    fn test_vec_pooling() {
        let fakepool = crate::memory::pool::FakePool::new(true);

        // Acquire an empty vector from the pool
        let mut vector = fakepool.acquiring().acquire::<Vec<u8>>().take();
        assert!(vector.is_empty());
        vector.reserve_exact(32);
        vector.extend_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);

        // Release the vec back into the pool
        fakepool.releasing().release(vector);

        // Check that the array is recycled
        let arrays = fakepool.try_into_array_allocations().unwrap();
        assert_eq!(arrays.len(), 1);
        assert_eq!(arrays[0].layout().size(), 32);
    }
}
