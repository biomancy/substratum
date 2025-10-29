use crate::memory::pool::core::TaggedLease;
use crate::memory::pool::{FromPool, IntoPool, PoolAcquire, PoolLease, PoolRelease, TryFromPool};
use crate::memory::{
    pool::{PoolAcquireHandle, PoolReleaseHandle}, ArrayAllocation, ObjectAllocation,
    Pool,
};
use std::alloc::Layout;
use std::any::TypeId;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

const FAKE_POOL_POISONED_MSG: &str =
    "FakePool mutex was poisoned. This is an unexpected and irrecoverable error.";

/// A fake memory pool for testing and demonstration purposes.
///
/// `FakePool` simulates the `Pool` interface but does not perform any actual pooling
/// or recycling of memory. Instead, it acts as a passthrough allocator. When asked for
/// an object, it always pretends to be empty. When an object is released, it is either
/// dropped or stored internally for inspection, depending on the configuration.
///
/// This is useful for testing application logic that uses the `substratum` pooling
/// API without depending on the complex behavior of a real, high-performance pool.
///
/// ### Behavior
/// - **Acquire Operations:** Always pretends that the cache is empty, requiring a new object to be
///   allocated.
/// - **Release Operations:** Can be configured to either drop released objects immediately
///   or save their raw allocations for later inspection via [`try_into_inner`](Self::try_into_inner).
///
/// ### Example
/// ```
/// # use substratum::memory::Pool;
/// # use substratum::memory::pool::{FakePool, PoolAcquire, PoolRelease};
/// // Create a pool that saves released allocations for inspection.
/// let pool = FakePool::new(true);
///
/// // Acquire a lease for a vector of bytes
/// let mut lease = pool.acquiring().acquire::<Vec<u8>>();
/// lease.reserve_exact(13);
///
/// // Release the lease back to the pool
/// pool.releasing().reclaim(lease);
///
/// // Inspect the allocations that were "released".
/// let (objects, arrays) = pool.try_into_inner().unwrap();
/// assert!(objects.is_empty());
/// assert_eq!(arrays.len(), 1);
/// assert_eq!(arrays[0].layout().size(), 13);
/// ```
#[derive(Debug, Default)]
pub struct FakePool {
    inner: Arc<Mutex<(Vec<ObjectAllocation>, Vec<ArrayAllocation>)>>,
    save_allocations: AtomicBool,
}

impl Clone for FakePool {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            save_allocations: AtomicBool::new(self.save_allocations.load(Ordering::Relaxed)),
        }
    }
}

impl FakePool {
    /// Creates a new `FakePool`.
    ///
    /// If `save_allocations` is `true`, the pool will store the raw allocations of
    /// released objects internally. If `false`, released objects are simply dropped.
    pub fn new(save_allocations: bool) -> Self {
        Self {
            inner: Arc::new(Mutex::new((Vec::new(), Vec::new()))),
            save_allocations: AtomicBool::new(save_allocations),
        }
    }

    /// Configure the pool to save all subsequent allocations.
    pub fn save_allocations(&self) -> &Self {
        self.save_allocations.store(true, Ordering::Relaxed);
        self
    }

    /// Configure the pool to discard all subsequent allocations.
    pub fn discard_allocations(&self) -> &Self {
        self.save_allocations.store(false, Ordering::Relaxed);
        self
    }

    /// Consumes the pool and returns the saved raw allocations.
    ///
    /// This method will only succeed if there are no other `Arc` clones of this `FakePool`
    /// instance. If other clones exist, it will fail and return `Err(self)`.
    /// This is the primary mechanism for tests to verify memory management behavior.
    pub fn try_into_inner(self) -> Result<(Vec<ObjectAllocation>, Vec<ArrayAllocation>), Self> {
        match Arc::try_unwrap(self.inner) {
            Ok(mutex) => Ok(mutex.into_inner().expect(FAKE_POOL_POISONED_MSG)),
            Err(inner) => Err(Self {
                inner,
                save_allocations: self.save_allocations,
            }),
        }
    }

    /// Consumes the pool and returns the saved object allocations.
    ///
    /// This method will only succeed if there are no other `Arc` clones of this `FakePool`
    /// instance and **no array allocations are saved**. Otherwise, it will fail and
    /// return `Err(self)`.
    pub fn try_into_object_allocations(self) -> Result<Vec<ObjectAllocation>, Self> {
        let save_allocations = self.save_allocations.load(Ordering::Relaxed);
        let (objects, arrays) = self.try_into_inner()?;
        if !arrays.is_empty() {
            return Err(Self {
                inner: Arc::new(Mutex::new((objects, arrays))),
                save_allocations: AtomicBool::new(save_allocations),
            });
        }
        Ok(objects)
    }

    /// Consumes the pool and returns the saved array allocations.
    ///
    /// This method will only succeed if there are no other `Arc` clones of this `FakePool`
    /// instance and **no object allocations are saved**. Otherwise, it will fail and
    /// return `Err(self)`.
    pub fn try_into_array_allocations(self) -> Result<Vec<ArrayAllocation>, Self> {
        let save_allocations = self.save_allocations.load(Ordering::Relaxed);
        let (objects, arrays) = self.try_into_inner()?;
        if !objects.is_empty() {
            return Err(Self {
                inner: Arc::new(Mutex::new((objects, arrays))),
                save_allocations: AtomicBool::new(save_allocations),
            });
        }
        Ok(arrays)
    }

    /// Acquires a lock on the pool's internal state, returning a locked object.
    fn locked(&self) -> LockedFakePool<'_> {
        LockedFakePool {
            guard: self.inner.lock().expect(FAKE_POOL_POISONED_MSG),
            save_allocations: self.save_allocations.load(Ordering::Relaxed),
        }
    }
}

impl Pool for FakePool {
    type ReleaseTxn<'txn> = LockedFakePool<'txn>;
    type AcquireTxn<'txn> = LockedFakePool<'txn>;
    type Leased<'a, T: 'a + IntoPool> = TaggedLease<'a, (), T>;

    fn releasing(&self) -> LockedFakePool<'_> {
        self.locked()
    }

    fn acquiring(&self) -> LockedFakePool<'_> {
        self.locked()
    }

    fn clear(&self) {
        let mut txn = self.locked();
        txn.guard.0.clear();
        txn.guard.1.clear();
    }

    fn optimize(&self) {
        let mut txn = self.locked();
        txn.guard.0.shrink_to_fit();
        txn.guard.1.shrink_to_fit();
    }
}

/// A handle representing a locked [`FakePool`].
///
/// This object holds a `MutexGuard` on the pool's internal state, ensuring that
/// all operations within its lifetime are atomic.
#[derive(Debug)]
pub struct LockedFakePool<'a> {
    guard: MutexGuard<'a, (Vec<ObjectAllocation>, Vec<ArrayAllocation>)>,
    save_allocations: bool,
}

/// A handle for releasing or acquiring objects from a locked [`FakePool`].
#[derive(Debug)]
pub struct LockedFakePoolHandle<'a, 'txn> {
    pool: &'a mut LockedFakePool<'txn>,
}

impl<'txn> PoolRelease for LockedFakePool<'txn> {
    type Leased<'a, T: 'a + IntoPool> = TaggedLease<'a, (), T>;
    fn release<T: IntoPool>(&mut self, obj: T) -> &mut Self {
        let mut handle = LockedFakePoolHandle { pool: self };
        obj.into_pool(&mut handle);
        self
    }

    fn reclaim<'a, T: IntoPool + 'a>(&mut self, lease: Self::Leased<'a, T>) -> &mut Self {
        let mut handle = LockedFakePoolHandle { pool: self };
        lease.take().into_pool(&mut handle);
        self
    }
}

impl<'a, 'txn> PoolReleaseHandle for LockedFakePoolHandle<'a, 'txn> {
    fn put_object(&mut self, _identity: TypeId, allocation: ObjectAllocation) {
        if self.pool.save_allocations {
            self.pool.guard.0.push(allocation);
        }
    }

    fn put_array(&mut self, _identity: TypeId, allocation: ArrayAllocation) {
        if self.pool.save_allocations {
            self.pool.guard.1.push(allocation);
        }
    }
    fn release<T: IntoPool>(&mut self, obj: T) {
        obj.into_pool(self);
    }
}

impl<'txn> PoolAcquire for LockedFakePool<'txn> {
    type Leased<'a, T: IntoPool + 'a> = TaggedLease<'a, (), T>;

    fn acquire<'a, T: FromPool + IntoPool + 'a>(&mut self) -> Self::Leased<'a, T> {
        let mut handle = LockedFakePoolHandle { pool: self };
        Self::Leased::new((), T::from_pool(&mut handle))
    }

    fn try_acquire<'a, T: TryFromPool + IntoPool + 'a>(&mut self) -> Option<Self::Leased<'a, T>> {
        let mut handle = LockedFakePoolHandle { pool: self };
        T::try_from_pool(&mut handle).map(|obj| Self::Leased::new((), obj))
    }
}

impl<'a, 'txn> PoolAcquireHandle for LockedFakePoolHandle<'a, 'txn> {
    fn take_object(&mut self, _: TypeId, _: Layout) -> Option<ObjectAllocation> {
        None
    }

    fn take_array(&mut self, _: TypeId, _: usize, _: usize) -> Option<ArrayAllocation> {
        None
    }

    fn acquire<T: FromPool>(&mut self) -> T {
        T::from_pool(self)
    }

    fn try_acquire<T: TryFromPool>(&mut self) -> Option<T> {
        T::try_from_pool(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::pool::{Pool, PoolAcquire, PoolRelease};

    #[test]
    fn try_into_inner() {
        let pool = FakePool::new(true);

        // try_into_inner will succeed only if there are no clones
        let clone = pool.clone();
        assert!(clone.try_into_inner().is_err());

        let (objects, arrays) = pool.try_into_inner().unwrap();
        assert!(objects.is_empty());
        assert!(arrays.is_empty());
    }

    #[test]
    fn release() {
        let pool = FakePool::new(false);

        pool.releasing().release(Vec::<u8>::with_capacity(8));
        pool.save_allocations();
        pool.releasing().release(Vec::<u8>::with_capacity(16));
        pool.discard_allocations();
        pool.releasing().release(Vec::<u8>::with_capacity(32));

        let arrays = pool.try_into_array_allocations().unwrap();
        assert_eq!(arrays.len(), 1);
        assert_eq!(arrays[0].element_size(), 1);
        assert_eq!(arrays[0].layout().size(), 16);
    }

    #[test]
    fn acquire() {
        let pool = FakePool::new(false);

        let mut lease = pool.acquiring().acquire::<Vec<u8>>();
        lease.reserve_exact(16);
        pool.releasing().reclaim(lease);

        // We should have any objects cached in the pool - trying to acquire will always fail
        let lease = pool.acquiring().try_acquire::<Vec<u8>>();
        assert!(lease.is_none());

        let (objects, arrays) = pool.try_into_inner().unwrap();
        assert!(objects.is_empty());
        assert!(arrays.is_empty());
    }

    #[test]
    fn clear_removes_all_allocations() {
        let pool = FakePool::new(true);
        pool.releasing()
            .release(Vec::<u8>::with_capacity(8))
            .release(Vec::<u8>::with_capacity(16));

        pool.clear();

        let (objects, arrays) = pool.try_into_inner().unwrap();
        assert!(objects.is_empty());
        assert!(arrays.is_empty());
    }

    #[test]
    fn optimize_does_not_panic() {
        let pool = FakePool::new(true);
        pool.releasing().release(Vec::<u8>::with_capacity(8));
        pool.optimize(); // Should not panic
    }
}
