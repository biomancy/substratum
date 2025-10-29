use crate::memory::pool::lifecycle::{FromPool, IntoPool, TryFromPool};
use std::ops::{Deref, DerefMut};

/// A "must-use" wrapper for an object acquired from a pool.
///
/// This wrapper acts as a "lease" on a pooled object. Its primary purpose is to prevent
/// the object's memory allocation from being accidentally leaked. Objects implementing
/// this trait should panic if they are dropped in debug builds and silently leak
/// their memory in release builds.
///
/// To correctly handle a [`PoolLease`] object, you must use one of two methods:
/// 1.  **[`PoolLease::take`]**: Consumes the wrapper and returns the inner object, making you
///     responsible for its eventual release. This is the "untracked" path.
/// 2.  **[`PoolRelease::reclaim`]**: Consumes the wrapper and returns the object
///     and its metadata to the pool in the most optimal way.
pub trait PoolLease<'a, T: 'a + IntoPool>: Deref<Target = T> + DerefMut {
    /// Consumes the lease, returning the underlying pooled object.
    ///
    /// After calling [`.take()`](PoolLease::take), you gain full ownership of the object.
    /// You are now responsible for manually returning it to the pool later using
    /// [`PoolRelease::release`]. Any performance benefits of a tracked return are forgone.
    fn take(self) -> T;
}

/// A transaction for releasing objects back to the pool.
///
/// A release transaction may lock the pool for its entire duration, making batch
/// operations significantly more efficient than releasing items one by one.
pub trait PoolRelease {
    /// The specific [`PoolLease`] item type associated with this transaction's pool.
    type Leased<'a, T: 'a + IntoPool>: PoolLease<'a, T>;

    /// Releases an untracked object back to the pool.
    fn release<T: IntoPool>(&mut self, obj: T) -> &mut Self;

    /// Reclaim a tracked [`PoolLease`] item back to the pool.
    ///
    /// This is often more efficient than releasing a raw object, as the [`PoolLease`]
    /// item may contain metadata that helps to optimize the release process.
    fn reclaim<'a, T: IntoPool + 'a>(&mut self, item: Self::Leased<'a, T>) -> &mut Self;
}

/// A transaction for acquiring objects from the pool.
///
/// An acquire transaction may lock the pool for its entire duration, making batch
/// operations significantly more efficient than acquiring items one by one.
pub trait PoolAcquire {
    /// The specific [`PoolLease`] item type associated with this transaction's pool.
    type Leased<'a, T: IntoPool + 'a>: PoolLease<'a, T>;

    /// **Infallibly** acquires a tracked object from the pool.
    ///
    /// This will use cached resources when possible and create new allocations if necessary.
    /// The returned [`PoolLease`] item must be explicitly handled to avoid a panic (debug) or a
    /// silent leak (release).
    #[must_use]
    fn acquire<'a, T: FromPool + IntoPool + 'a>(&mut self) -> Self::Leased<'a, T>;

    /// **Fallibly** acquires a tracked object using only cached resources.
    ///
    /// This will return `None` if creating the object would require a new allocation.
    /// The returned [`PoolLease`] item, if successful, must be explicitly handled to avoid a
    /// panic (debug) or a silent leak (release).
    #[must_use]
    fn try_acquire<'a, T: TryFromPool + IntoPool + 'a>(&mut self) -> Option<Self::Leased<'a, T>>;
}

/// The main user-facing trait for an object memory pool.
pub trait Pool: Send + Sync + 'static {
    /// The release transaction type for this pool.
    type ReleaseTxn<'txn>: PoolRelease;

    /// The acquire transaction type for this pool.
    type AcquireTxn<'txn>: PoolAcquire;

    /// The "must-use" wrapper type for objects leased from this pool.
    type Leased<'a, T: 'a + IntoPool>: PoolLease<'a, T>;

    /// Begins a transaction for releasing objects back to the pool.
    ///
    /// A transaction provides an efficient way to perform batch operations. The underlying
    /// pool is typically locked for the entire lifetime of the returned transaction object,
    /// avoiding the overhead of repeated locking for individual operations.
    fn releasing(&self) -> Self::ReleaseTxn<'_>;

    /// Begins a transaction for acquiring objects from the pool.
    ///
    /// A transaction provides an efficient way to perform batch operations. The underlying
    /// pool is typically locked for the entire lifetime of the returned transaction object,
    /// avoiding the overhead of repeated locking for individual operations.
    fn acquiring(&self) -> Self::AcquireTxn<'_>;

    /// Clears the pool, dropping all cached allocations.
    fn clear(&self);

    /// Optimizes the pool's internal state.
    ///
    /// The exact behavior is implementation-defined but may include actions like
    /// removing empty internal core or shrinking excess capacity.
    fn optimize(&self);
}
