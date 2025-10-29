use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::sync::PoisonError;

#[derive(Debug)]
pub enum ViewError {
    /// An index-based access was out of bounds.
    IndexOutOfBounds(usize),

    /// A synchronization lock (e.g., RwLock, Mutex) was poisoned.
    ///
    /// This indicates that another thread panicked while holding the lock.
    LockPoisoned(String),

    /// A non-blocking `try_lock` operation failed, as the
    /// resource was already locked.
    LockBusy,

    /// An external, boxed error.
    ///
    /// This serves as an escape hatch for arbitrary errors
    /// that don't fit other categories.
    Other(Box<dyn std::error::Error + Send + Sync>),
}

impl Display for ViewError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ViewError::IndexOutOfBounds(i) => write!(f, "Index {i} is out of bounds"),
            ViewError::LockPoisoned(s) => write!(f, "Synchronization lock poisoned: {s}"),
            ViewError::LockBusy => write!(f, "Resource is currently locked and try_lock failed"),
            ViewError::Other(e) => Display::fmt(e, f),
        }
    }
}

impl Error for ViewError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ViewError::Other(e) => Some(e.as_ref()),
            _ => None,
        }
    }
}

impl From<Box<dyn Error + Send + Sync>> for ViewError {
    fn from(e: Box<dyn Error + Send + Sync>) -> Self {
        ViewError::Other(e)
    }
}

impl<T> From<PoisonError<T>> for ViewError {
    fn from(e: PoisonError<T>) -> Self {
        ViewError::LockPoisoned(e.to_string())
    }
}
