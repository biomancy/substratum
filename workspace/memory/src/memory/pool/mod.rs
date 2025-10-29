//! Provides the core traits and concrete implementations for memory pools.
//!
//! This module defines a toolkit for efficient memory recycling by caching heap objects
//! and dynamic arrays, aiming to minimize allocation overhead in performance-critical
//! applications.
//!
//! ## Design Philosophy
//!
//! The library's design is centered around two key concepts: an object's **Lifecycle** and the
//! **Pools** that manage it.
//!
//! ### The Lifecycle Traits
//!
//! A type becomes "poolable" by implementing a set of lifecycle traits:
//!
//! - **[`IntoPool`]**: Defines how to deconstruct an object and return its underlying memory
//!   allocations to a pool. Assigns a logical `Identity`, which allows related types (e.g.,
//!   `MyObject` and `MyObjectBuilder`) to be treated as a single poolable type.
//! - **[`FromPool`]** & **[`TryFromPool`]**: Define how to construct an object by reusing allocations
//!   from a pool.
//!
//! An object can implement `IntoPool` without `FromPool`, but not the reverse. This allows types
//! with complex invariants to be deconstructed and recycled, while their construction might be
//! handled exclusively by a separate builder type that implements `FromPool`.
//!
//! ## Ready-to-Use Implementations
//!
//! This module re-exports several ready-to-use memory pool implementations:
//! * [`FakePool`] - a simple pool that always allocates new objects, useful for testing.
//! * [`Loom`] - hierarchical pool that allows threads to reuse arrays having compatible layouts
//!   and steal array allocations from other threads.
//!
//! See the documentation for each type for more details on their usage and performance characteristics.
//! ---
//! ## Example
//!
//! ```
// use substratum::memory::pool::{MutexPool, Pool, StealingAffinityPool};
// use std::sync::Arc;
//
// // 1. Create a PoolCore algorithm instance.
// let core_pool = StealingAffinityPool::default();
//
// // 2. Wrap it in a thread-safe Pool handle. Arc is used for sharing across threads.
// let pool = Arc::new(MutexPool::from_pool_core(core_pool));
//
// // 3. Acquire a pooled object. The returned `Guard` manages the object's lifetime.
// let mut my_vec_guard = pool.acquire::<Vec<u8>>();
//
// // 4. Use the object as needed.
// my_vec_guard.extend_from_slice(&[1, 2, 3, 4]);
// assert_eq!(*my_vec_guard, vec![1, 2, 3, 4]);
//!
//! // 5. The Vec<u8>'s memory is automatically returned to the pool when `my_vec_guard`
//! //    is dropped at the end of the scope.
//! ```

mod adapters;
pub mod core;
mod fake_pool;
mod lifecycle;
mod loom;
mod pool;

pub use fake_pool::FakePool;
pub use lifecycle::{FromPool, IntoPool, PoolAcquireHandle, PoolReleaseHandle, TryFromPool};
pub use pool::{Pool, PoolAcquire, PoolLease, PoolRelease};
