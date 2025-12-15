use super::error::MultiLockError;
use super::lock_builder::MultiLockBuilder;
use super::multi_guard::{MultiGuard, ReadGuard};
use crate::WriteGuard;

/// Defines a resources that knows how to register locks required for its shared read-only access.
pub trait RegisterReadLocks {
    /// Register all locks required for read access to this resource into the
    /// provided [`MultiLockBuilder`].
    ///
    /// This method follows the visitor pattern, expecting the implementor to
    /// recursively delegate the registration of locks to its contained
    /// [`RegisterReadLocks`] resources, until reaching the actual locks that
    /// implement the registration logic (e.g., [`lock_api::Mutex`]).
    fn register_read_locks<'a, 'b: 'a>(&'b self, to: &'a mut MultiLockBuilder<'b>);
}

/// Defines a shared resource that requires one or more locks for access.
pub trait ReadLockable: RegisterReadLocks {
    /// The type of data that can be accessed once all required locks are held.
    type Data;

    /// Given a [`MultiGuard`] proof that all required locks are held, provide read-only
    /// access to the underlying data.
    fn read_with_guard<'a>(
        &'a self,
        proof: &'a MultiGuard<'a>,
    ) -> Result<ReadGuard<'a, Self::Data>, MultiLockError>;
}

/// Defines a resource that knows how to register locks required for its exclusive access.
pub trait RegisterWriteLocks {
    /// Register all locks required for exclusive access to this resource into the
    /// provided [`MultiLockBuilder`].
    ///
    /// This method follows the visitor pattern, expecting the implementor to
    /// recursively delegate the registration of locks to its contained
    /// [`WriteLockable`] resources, until reaching the actual locks that
    /// implement the registration logic (e.g., [`lock_api::Mutex`]).
    fn register_write_locks<'a, 'b: 'a>(&'b self, to: &'a mut MultiLockBuilder<'b>);
}

/// Defines an exclusive resource that require one or more locks for access.
pub trait WriteLockable: RegisterWriteLocks {
    /// The type of data that can be accessed once all required locks are held.
    type Data;

    /// Given a [`MultiGuard`] proof that all required locks are held, provide write
    /// access to the underlying data.
    fn write_with_guard<'a>(
        &'a self,
        proof: &'a MultiGuard<'a>,
    ) -> Result<WriteGuard<'a, Self::Data>, MultiLockError>;
}
