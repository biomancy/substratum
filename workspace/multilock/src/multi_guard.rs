use super::error::MultiLockError;
use super::lock::{LockId, UnlockOnDrop};
use std::cell::{Ref, RefCell, RefMut};
use std::collections::{BTreeMap, BTreeSet};
use std::ops::{Deref, DerefMut};

/// A RAII guard that provides read-only access to data protected by a [`MultiGuard`].
pub struct ReadGuard<'a, T> {
    data: &'a T,
    _token: ReadLockProof<'a>,
}

impl<'a, T> Deref for ReadGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.data
    }
}

impl<'a, T> ReadGuard<'a, T> {
    /// Map the underlying data to a new type, preserving the read lock.
    pub fn map<U, F>(guard: Self, f: F) -> ReadGuard<'a, U>
    where
        F: FnOnce(&T) -> &U,
    {
        ReadGuard {
            data: f(guard.data),
            _token: guard._token,
        }
    }

    /// Fallibly map the underlying data to a new type, preserving the read lock.
    pub fn try_map<U, F, E>(guard: Self, f: F) -> Result<ReadGuard<'a, U>, E>
    where
        F: FnOnce(&T) -> Result<&U, E>,
    {
        Ok(ReadGuard {
            data: f(guard.data)?,
            _token: guard._token,
        })
    }
}

/// A RAII guard that provides write access to data protected by a [`MultiGuard`].
pub struct WriteGuard<'a, T> {
    data: &'a mut T,
    _token: WriteLockProof<'a>,
}

impl<'a, T> Deref for WriteGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.data
    }
}

impl<'a, T> DerefMut for WriteGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.data
    }
}

impl<'a, T> WriteGuard<'a, T> {
    /// Map the underlying data to a new type, preserving the write lock.
    pub fn map<U, F>(guard: Self, f: F) -> WriteGuard<'a, U>
    where
        F: FnOnce(&mut T) -> &mut U,
    {
        WriteGuard {
            data: f(guard.data),
            _token: guard._token,
        }
    }

    /// Fallibly map the underlying data to a new type, preserving the write lock.
    pub fn try_map<U, F, E>(guard: Self, f: F) -> Result<WriteGuard<'a, U>, E>
    where
        F: FnOnce(&mut T) -> Result<&mut U, E>,
    {
        Ok(WriteGuard {
            data: f(guard.data)?,
            _token: guard._token,
        })
    }
}

/// A smart token proving read access to a resource.
#[derive(Debug)]
pub enum ReadLockProof<'a> {
    Simple,
    WithBorrow(Ref<'a, ()>),
}

impl<'a> ReadLockProof<'a> {
    /// Attach the token to some data, producing a `SharedMultiGuard`.
    pub fn attach<T>(self, data: &'a T) -> ReadGuard<'a, T> {
        ReadGuard { data, _token: self }
    }
}

/// A smart token proving write access to a resource.
#[derive(Debug)]
pub enum WriteLockProof<'a> {
    WithBorrow(RefMut<'a, ()>),
}

impl<'a> WriteLockProof<'a> {
    /// Attach the token to some data, producing an `ExclusiveMultiGuard`.
    pub fn attach<T>(self, data: &'a mut T) -> WriteGuard<'a, T> {
        WriteGuard { data, _token: self }
    }
}

/// A RAII token that proves that multiple locks are held simultaneously.
///
/// A `MultiGuard` is created by a [`MultiLockBuilder`] and ensures that all
/// acquired locks are held until the `MultiGuard` is dropped. All locks
/// are released in the reverse order of acquisition when the guard goes
/// out of scope.
///
/// Lock status can be queried by using `is_locked_shared()` and `is_locked_exclusive()`.
/// Returned tokens must be held as long as any references based on these proofs are alive.
pub struct MultiGuard<'a> {
    #[allow(dead_code, dyn_drop)]
    guards: Vec<UnlockOnDrop<'a>>,
    shared: BTreeSet<LockId<'a>>,
    exclusive: BTreeMap<LockId<'a>, RefCell<()>>,
}

impl<'a> MultiGuard<'a> {
    #[allow(dyn_drop)]
    pub(crate) fn new(
        guards: Vec<UnlockOnDrop<'a>>,
        shared: BTreeSet<LockId<'a>>,
        exclusive: BTreeMap<LockId<'a>, RefCell<()>>,
    ) -> Self {
        MultiGuard {
            guards,
            shared,
            exclusive,
        }
    }

    pub fn read_proof(&'a self, lock: LockId<'a>) -> Result<ReadLockProof<'a>, MultiLockError> {
        if self.shared.contains(&lock) {
            return Ok(ReadLockProof::Simple);
        }

        if let Some(cell) = self.exclusive.get(&lock) {
            let borrow = cell.try_borrow().map_err(MultiLockError::from)?;
            return Ok(ReadLockProof::WithBorrow(borrow));
        }

        Err(MultiLockError::LockNotHeld)
    }

    pub fn write_proof(&'a self, lock: LockId<'a>) -> Result<WriteLockProof<'a>, MultiLockError> {
        self.exclusive
            .get(&lock)
            .ok_or(MultiLockError::LockNotHeld)
            .and_then(|cell| cell.try_borrow_mut().map_err(MultiLockError::from))
            .map(WriteLockProof::WithBorrow)
    }
}

impl<'a> Drop for MultiGuard<'a> {
    fn drop(&mut self) {
        // Guards must be dropped in reverse order of acquisition.
        for guard in self.guards.drain(..).rev() {
            drop(guard);
        }
    }
}
