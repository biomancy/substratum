use std::cell::{BorrowError, BorrowMutError};
use std::error::Error;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};

/// Represents errors that can occur during multi-lock operations.
#[derive(Debug)]
pub enum MultiLockError {
    /// A lock has been poisoned. This typically happens when a thread
    /// panics while holding an exclusive lock.
    Poisoned,

    /// The requested lock is not held by the `MultiGuard`.
    LockNotHeld,

    /// A non-blocking [`crate::MultiLockBuilder::try_lock`] operation failed because the
    /// lock was already held by another thread.
    TryLockFailed,

    /// Cannot acquire read access because the resource is already locked mutably.
    ///
    /// This error originates from `RefCell::try_borrow` within the `MultiGuard`
    /// when attempting to read from an exclusively (write) locked resource
    /// that is already borrowed mutably by the current thread.
    AlreadyLockedMutably(BorrowError),

    /// Cannot acquire an exclusive access because the resource is already locked as shared.
    ///
    /// This error originates from `RefCell::try_borrow_mut` within the `MultiGuard`
    /// when attempting to get an exclusive token to a resource that is already
    /// borrowed (either shared or exclusively).
    AlreadyLocked(BorrowMutError),

    /// A generic, boxed error for other, less common failures.
    Other(Box<dyn Error + Send + Sync>),
}

impl From<BorrowError> for MultiLockError {
    /// Converts a `BorrowError` into `LockError::AlreadyLockedMutably`.
    fn from(err: BorrowError) -> Self {
        MultiLockError::AlreadyLockedMutably(err)
    }
}

impl From<BorrowMutError> for MultiLockError {
    /// Converts a `BorrowMutError` into `LockError::AlreadyLocked`.
    fn from(err: BorrowMutError) -> Self {
        MultiLockError::AlreadyLocked(err)
    }
}

impl Display for MultiLockError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            MultiLockError::Poisoned => write!(f, "A lock has been poisoned"),
            MultiLockError::TryLockFailed => write!(f, "A non-blocking lock attempt failed"),
            MultiLockError::AlreadyLockedMutably(_) => {
                write!(
                    f,
                    "Cannot acquire read access; resource is already locked mutably"
                )
            }
            MultiLockError::AlreadyLocked(_) => {
                write!(f, "Cannot acquire write access; resource is already locked")
            }
            MultiLockError::Other(err) => write!(f, "An external error occurred: {}", err),
            MultiLockError::LockNotHeld => {
                write!(f, "The requested lock is not held by this MultiGuard")
            }
        }
    }
}

impl Error for MultiLockError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            MultiLockError::AlreadyLockedMutably(err) => Some(err),
            MultiLockError::AlreadyLocked(err) => Some(err),
            MultiLockError::Other(err) => Some(err.as_ref()),
            _ => None,
        }
    }
}
