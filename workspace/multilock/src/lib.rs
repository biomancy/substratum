/// Deadlock-Safe, Composable Multi-Lock Acquisition
///
/// `multilock` provides a composable system for acquiring multiple locks
/// simultaneously, without the risk of a deadlock.
///
/// ## The Problem: Deadlock
///
/// In a multi-threaded system, "AB-BA" deadlocks are a common and difficult
/// problem.
/// - Thread 1 locks `A`, then tries to lock `B`.
/// - Thread 2 locks `B`, then tries to lock `A`.
/// - Both threads are now blocked, waiting for a resource held by the other.
///   This is a deadlock.
///
/// ## The Solution: Canonical Lock-Order
///
/// `multilock` solves this by enforcing a **canonical acquisition order**.
/// All locks are assigned a stable, unique [`LockId`] (e.g., based on their memory
/// address). Before acquiring any locks, a [`MultiLockBuilder`] collects all
/// lock requests (shared or exclusive) and sorts them by their [`LockId`].
///
/// It then acquires the locks, one by one, *in this sorted order*.
///
/// Because all threads will *always* attempt to acquire locks in the same
/// globally-consistent order (e.g., always `A` then `B`), the "circular wait"
/// condition for a deadlock is broken.
///
/// ## Core Components
///
/// 1.  [`ReadLock`] / [`WriteLock`] traits represent raw synchronization primitives
///     that can be registered for simultaneous acquisition in a [`MultiLockBuilder`].
/// 2.  [`MultiLockBuilder`] The entry point. Users register all required in this builder.
/// 3.  [`ReadLockable`] / [`WriteLockable`]: Abstractions that allow [`MultiLockBuilder`]
///     to recursively collect locks from any types that require locking to produce a result.
/// 4.  [`MultiGuard`]: The RAII guard returned by [`MultiLockBuilder::lock`]. It doesn't
///     hold any data itself, and instead serves as a proof that all locks have been acquired.
///
/// Finally, after acquiring a [`MultiGuard`], users can safely access the locked data via
/// [`ReadLockable::read_with_guard`] or [`WriteLockable::write_with_guard`].
///
/// ## Limitations
/// *  `multilock` supports only flat collections of locks. I.e., it's impossible
///    to multilock data hidden inside other locked data structures.
/// *  There is a runtime cost associated with access to each lock.
///     * Each request of a proof of lock possession requires a lookup in a binary tree.
///     * Write locks require runtime borrow checking by [`std::cell::RefCell`]. Generally,
///       two objects might depend on a common exclusive resource, but access it at different times.
///       This is a valid use case, and keeping track of live borrows require to prove that this
///       access is safe.
///     * Locking and unlocking are done via dyn traits, which adds a vtable lookup overhead
///       paid twice per lock (once for locking, once for unlocking).
///
mod lock;

mod error;
mod impls;
mod lock_builder;
mod lockable;
mod multi_guard;

pub use error::MultiLockError;
pub use lock::{LockId, ReadLock, Unlock, WriteLock};
pub use lock_builder::MultiLockBuilder;
pub use lockable::{ReadLockable, RegisterReadLocks, RegisterWriteLocks, WriteLockable};
pub use multi_guard::{MultiGuard, ReadGuard, ReadLockProof, WriteGuard, WriteLockProof};
