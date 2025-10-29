//! Provides foundational components for memory management.
//!
//! This module serves as the primary entry point for `substratum`'s memory
//! management features:
//! * [`pool.md`]: A recursive memory pool designed for recycling object and array allocations.
//! * ...: More features will be developed in the future.
//!
//! * [`Pool`]: The main user-facing, thread-safe trait for the memory pool.
//! * [`ObjectAllocation`]: A handle to a raw, fixed-size memory block suitable for
//!   a single object.
//! * [`ArrayAllocation`]: A handle to a raw, growable memory block suitable for
//!   backing array-like structures such as `Vec<T>`.

mod allocation;
pub mod pool;

pub use allocation::{ArrayAllocation, ObjectAllocation};
pub use pool::Pool;
