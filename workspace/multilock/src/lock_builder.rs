use super::error::MultiLockError;
use super::lock::{LockId, ReadLock, Unlock, UnlockOnDrop, WriteLock};
use super::lockable::{RegisterReadLocks, RegisterWriteLocks};
use super::multi_guard::MultiGuard;
use std::cell::RefCell;
use std::collections::btree_map::Entry;
use std::collections::{BTreeMap, BTreeSet};

enum LockRequest<'a> {
    /// A request for a shared (read) lock.
    Shared(&'a dyn ReadLock),
    /// A request for an exclusive (write) lock.
    Exclusive(&'a dyn WriteLock),
}

/// A builder for acquiring multiple locks simultaneously in a deadlock-safe manner.
///
/// # Deadlock Prevention
///
/// The key to deadlock prevention is that [`MultiLockBuilder`] stores requests in a
/// `BTreeMap` keyed by `LockId` (which is normally derived from the lock's memory address).
/// When `lock()` is called, it iterates over the `BTreeMap`, acquiring locks
/// in a stable, globally consistent order. This strategy fundamentally
/// breaks the "circular wait" condition required for a deadlock.
///
/// For example, if Thread 1 requests `lock(A)` then `lock(B)`, and Thread 2
/// requests `lock(B)` then `lock(A)`, a deadlock can occur.
///
/// With [`MultiLockBuilder`], both threads would build a list:
/// * Thread 1: `add(A)`, `add(B)`
/// * Thread 2: `add(B)`, `add(A)`
///
/// Assuming `Id(A) < Id(B)`, both threads' [`MultiLockBuilder`] internals will
/// store the requests in the order `[A, B]`. When `lock()` is called,
/// both threads will attempt to acquire `A` first, then `B`. One thread
/// will succeed, and the other will block, but no deadlock will occur.
pub struct MultiLockBuilder<'a> {
    locks: BTreeMap<LockId<'a>, LockRequest<'a>>,
}

impl<'a> Default for MultiLockBuilder<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> MultiLockBuilder<'a> {
    /// Creates a new, empty [`MultiLockBuilder`].
    pub fn new() -> Self {
        Self {
            locks: BTreeMap::new(),
        }
    }

    /// Adds all locks required to lock a [`SharedLockable`] object.
    pub fn with_read<T: RegisterReadLocks>(mut self, lockable: &'a T) -> Self {
        lockable.register_read_locks(&mut self);
        self
    }

    /// Adds all locks required to lock a [`ExclusiveLockable`] object.
    pub fn with_write<T: RegisterWriteLocks>(mut self, lockable: &'a T) -> Self {
        lockable.register_write_locks(&mut self);
        self
    }

    /// Adds a read lock to the builder.
    ///
    /// The caller provides the `LockId` for the lock being added. The same `LockId` must be
    /// used when accessing the lock later.
    ///
    /// If the lock was previously requested as a read or a write lock, this call has no effect.
    pub fn add_read_lock<L: ReadLock + 'a>(&mut self, id: LockId<'a>, lock: &'a L) -> &mut Self {
        match self.locks.entry(id) {
            // If any lock (read or write) is already registered, do nothing.
            // A write lock subsumes a read request.
            // A read lock is idempotent.
            Entry::Occupied(_) => {}
            // If no lock is registered, add this read lock request.
            Entry::Vacant(x) => {
                x.insert(LockRequest::Shared(lock));
            }
        }
        self
    }

    /// Adds a write lock to the builder.
    ///
    /// The caller provides the `LockId` for the lock being added. The same `LockId` must be
    /// used when accessing the lock later.
    ///
    /// If this lock was previously requested as a read lock, it is upgraded to a write lock.
    /// If it was already requested as a write lock, this call has no effect.
    pub fn add_write_lock<L: WriteLock + 'a>(&mut self, id: LockId<'a>, lock: &'a L) -> &mut Self {
        match self.locks.entry(id) {
            // A lock request already exists.
            Entry::Occupied(mut existing) => {
                match existing.get() {
                    // It's already a write lock, so we're done.
                    LockRequest::Exclusive(_) => {}
                    // It's a read lock; upgrade it to a write lock.
                    LockRequest::Shared(_) => {
                        existing.insert(LockRequest::Exclusive(lock));
                    }
                }
            }
            // No lock request exists yet; insert a new write lock request.
            Entry::Vacant(x) => {
                x.insert(LockRequest::Exclusive(lock));
            }
        }
        self
    }

    /// Acquires all requested locks and returns a `MultiGuard`.
    ///
    /// This method will block the current thread until all locks are
    /// successfully acquired. Locks are acquired in the canonical order
    /// of their `LockId`s to prevent deadlocks.
    ///
    /// The returned `MultiGuard` holds all acquired locks and will
    /// release them when it is dropped.
    pub fn lock(&self) -> Result<MultiGuard<'a>, MultiLockError> {
        let mut guards: Vec<UnlockOnDrop> = Vec::new();
        let mut shared: BTreeSet<LockId> = BTreeSet::new();
        let mut exclusive: BTreeMap<LockId, RefCell<()>> = BTreeMap::new();

        for (id, lock_type) in &self.locks {
            match lock_type {
                LockRequest::Shared(lock) => {
                    lock.lock_read();
                    shared.insert(*id);
                    guards.push(UnlockOnDrop(*lock as &dyn Unlock));
                }
                LockRequest::Exclusive(lock) => {
                    lock.lock_write();
                    exclusive.insert(*id, RefCell::new(()));
                    guards.push(UnlockOnDrop(*lock as &dyn Unlock));
                }
            }
        }

        Ok(MultiGuard::new(guards, shared, exclusive))
    }

    /// Attempts to acquire all requested locks without blocking.
    ///
    /// This method attempts to acquire all locks in the canonical, sorted
    /// order of their `LockId`s.
    ///
    /// If *all* locks are acquired successfully, it returns a `MultiGuard`
    /// that holds all the locks.
    ///
    /// If *any* lock cannot be acquired immediately (i.e., it is
    /// contended), this method immediately returns `Err(LockError::TryLockFailed)`.
    /// Any locks that were successfully acquired *before* the contended lock
    /// are automatically released. This ensures that a failed `try_lock`
    /// holds no locks.
    pub fn try_lock(&self) -> Result<MultiGuard<'a>, MultiLockError> {
        let mut guards: Vec<UnlockOnDrop> = Vec::new();
        let mut shared: BTreeSet<LockId> = BTreeSet::new();
        let mut exclusive: BTreeMap<LockId, RefCell<()>> = BTreeMap::new();

        for (id, lock_type) in &self.locks {
            match lock_type {
                LockRequest::Shared(lock) => {
                    if !lock.try_lock_read() {
                        // Release all previously acquired locks in the reverse order.
                        guards.drain(..).rev().for_each(drop);
                        return Err(MultiLockError::TryLockFailed);
                    }
                    shared.insert(*id);
                    guards.push(UnlockOnDrop(*lock as &dyn Unlock));
                }
                LockRequest::Exclusive(lock) => {
                    if !lock.try_lock_write() {
                        // Release all previously acquired locks in the reverse order.
                        guards.drain(..).rev().for_each(drop);
                        return Err(MultiLockError::TryLockFailed);
                    }
                    exclusive.insert(*id, RefCell::new(()));
                    guards.push(UnlockOnDrop(*lock as &dyn Unlock));
                }
            }
        }

        Ok(MultiGuard::new(guards, shared, exclusive))
    }

    /// Creates a `MultiGuard` assuming all required locks are already held.
    ///
    /// # Safety
    ///
    /// The caller **must** guarantee that all locks registered with this
    /// [`MultiLockBuilder`] have already been acquired and will remain held
    /// for the entire lifetime `'a`.
    ///
    /// The returned `MultiGuard` will **not** contain any drop guards,
    /// so it will not release any locks when dropped. The caller is
    /// solely responsible for managing the lifetime of the locks.
    pub unsafe fn assume_locked(&self) -> MultiGuard<'a> {
        let mut shared: BTreeSet<LockId> = BTreeSet::new();
        let mut exclusive: BTreeMap<LockId, RefCell<()>> = BTreeMap::new();

        for (id, lock_type) in &self.locks {
            match lock_type {
                LockRequest::Shared(_) => {
                    shared.insert(*id);
                }
                LockRequest::Exclusive(_) => {
                    exclusive.insert(*id, RefCell::new(()));
                }
            }
        }

        MultiGuard::new(Vec::new(), shared, exclusive)
    }
}
