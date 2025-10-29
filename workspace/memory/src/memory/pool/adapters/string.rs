use crate::memory::pool::lifecycle::{
    FromPool, IntoPool, PoolAcquireHandle, PoolReleaseHandle, TryFromPool,
};

pub struct UTF8Byte;

impl IntoPool for String {
    type Identity = UTF8Byte;
    fn into_pool(self, handle: &mut impl PoolReleaseHandle) {
        self.into_bytes().into_pool(handle);
    }
}

impl FromPool for String {
    fn from_pool(handle: &mut impl PoolAcquireHandle) -> Self {
        Self::try_from_pool(handle).unwrap_or_default()
    }
}

impl TryFromPool for String {
    fn try_from_pool(handle: &mut impl PoolAcquireHandle) -> Option<Self> {
        let buffer = Vec::try_from_pool(handle)?;
        debug_assert!(buffer.is_empty());

        // SAFETY: The buffer is empty and thus is guaranteed to be valid UTF-8.
        unsafe { Some(String::from_utf8_unchecked(buffer)) }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::memory::pool::{PoolAcquire, PoolLease, PoolRelease};
    use crate::memory::Pool;
    #[test]
    fn test_string_pooling() {
        let fakepool = crate::memory::pool::FakePool::new(true);

        // Acquire an empty string from the pool
        let mut string = fakepool.acquiring().acquire::<String>().take();
        assert!(string.is_empty());
        string.reserve_exact(16);
        string.push_str("Hello world!");

        // Release the string back into the pool
        fakepool.releasing().release(string);

        // Check that the string array is recycled
        let arrays = fakepool.try_into_array_allocations().unwrap();
        assert_eq!(arrays.len(), 1);
        assert_eq!(arrays[0].layout().size(), 16);
    }
}
