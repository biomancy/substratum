/// A stable, unique identifier for a lock.
///
/// This ID is used to sort lock requests, ensuring a canonical order
/// of acquisition and thus preventing AB-BA deadlocks. This is typically
/// derived from the memory address of the underlying lock primitive
/// (e.g., [`lock_api::Mutex`] or [`lock_api::RwLock`]).
///
/// Users must ensure that the identifier remains constant for the
/// lifetime of the lock; otherwise, deadlocks may occur or locked data
/// may become inaccessible.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LockId<'a> {
    id: usize,
    _marker: std::marker::PhantomData<&'a ()>,
}

impl<'a> LockId<'a> {
    /// Creates a new `LockId` from the memory address pointed to by `ptr`.
    pub fn from_ptr<T>(ptr: *const T) -> Self {
        Self {
            id: ptr.addr(),
            _marker: std::marker::PhantomData,
        }
    }
}

/// A base trait to perform unlocking operations.
pub trait Unlock {
    /// Unlock a previously acquired lock.
    ///
    /// # Safety
    ///
    /// Callers must ensure that the lock is currently held by the calling thread.
    /// Some lock implementations may exhibit undefined behavior if this is not the case.
    unsafe fn unlock(&self);
}

/// A trait for locks that can be acquired for shared read-only access.
pub trait ReadLock: Unlock {
    fn lock_read(&self);

    fn try_lock_read(&self) -> bool;
}

/// A trait for locks that can be acquired for exclusive (mutable) access.
pub trait WriteLock: Unlock {
    fn lock_write(&self);

    fn try_lock_write(&self) -> bool;
}

/// A wrapper that unlocks a lock when dropped.
pub struct UnlockOnDrop<'a>(pub &'a dyn Unlock);
impl<'a> Drop for UnlockOnDrop<'a> {
    fn drop(&mut self) {
        unsafe { self.0.unlock() };
    }
}
