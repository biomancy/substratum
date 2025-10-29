//! # `vista`: A Composable Abstraction for Data Access
//!
//! This crate provides a flexible, composable abstraction for data access.
//! Its purpose is to abstract the *means* of accessing data, allowing
//! users to build powerful APIs that are agnostic to the underlying storage mechanism.
//!
//! The core pattern is "callback-based access" (or "lending"). Instead of
//! returning a reference, which is often impossible due to data ownership
//! rules (e.g., when data is inside an `RwLock`), this crate provides traits
//! that *lend* a reference to a user-supplied closure.
//!
//! ## Solved Problems
//!
//! * **Interior Mutability:** Access data inside `RwLock` or `Mutex`
//!   transparently, without exposing locking details to the user.
//! * **Storage Agnosticism:** Write code that works whether the data is
//!   behind an `Arc`, `Arc<RwLock>`, or a custom access primitive.
//! * **Computed Values:** Provide "views" of computed data (e.g., `vec.len()`)
//!   that feel like direct fields.
//! * **Fallible Access:** Cleanly handle access that might fail (e.g.,
//!   indexing a `Vec`).
//!
//! ## 🏛️ Architecture
//!
//! * [`View`]: The fundamental trait for safe, immutable access.
//! * [`ViewMut`]: The trait for safe, *mutable* access.
//!
//! **Note:** `ViewMut` does **not** implement `View`. This is a deliberate
//! design choice to prevent performance traps. `ViewMut` often implies an expensive
//! *exclusive lock*, while `View` implies a cheap *shared lock*. An explicit
//! bridge, [`bridges::Freeze`], is provided for cases where this conversion
//! is necessary.
//!
//! * [`UnsafeView`] / [`UnsafeViewMut`]: Separate "escape hatch" traits
//!   for performance-critical code. They operate on raw pointers and
//!   require the caller to uphold all synchronization guarantees.
//!
//! The [`ViewExt`] and [`ViewMutExt`] traits provide a fluent, chainable
//! API via combinators like `.map()`, `.try_map()`, and `.compute()`.
//!
//! ## Example
//!
//! ```rust
//! # use std::sync::{Arc, RwLock};
//! # use vista::{ArcRwLockView, View, ViewError, ViewMut, ViewMutExt};
//!
//! // --- Define application data ---
//! struct User {
//!  id: u64,
//!  name: String,
//!  scores: Vec<u32>,
//! }
//!
//! struct AppState {
//!  users: Vec<User>,
//! }
//!
//! // 1. Create the root data structure, protected by a lock.
//! let state = Arc::new(RwLock::new(AppState {
//!  users: vec![User {
//!      id: 1,
//!      name: "Alice".to_string(),
//!      scores: vec![10, 20, 30],
//!  }],
//! }));
//!
//! // 2. Create the base view. This is cheap and just clones the Arc.
//! let state_view = ArcRwLockView::new(state);
//!
//! // 3. Create a mutable view to the second score.
//! //    Note: This chain just creates nested structs; no locks are taken yet.
//! let mut_view = state_view
//!  .clone() // Clone the view, not the data
//!  .try_map_mut(|app_state| app_state.users.get_mut(0).ok_or(ViewError::IndexOutOfBounds(0)))
//!  .map_mut(|user| &mut user.scores)
//!  .try_map_mut(|scores| {
//!      scores.get_mut(1).ok_or(ViewError::IndexOutOfBounds(1))
//!  });
//!
//! // 4. Mutate the score. The RwLock::write() lock is taken here,
//! //    the closure is executed, and the lock is released.
//! mut_view.view_mut(&mut |score| {
//!  *score = 25;
//!  Ok(())
//! }).expect("Failed to mutate the score");
//!
//! // 6. Verify the mutation. The RwLock::read() lock is taken here.
//! //    the closure is executed, and the lock is released.
//! state_view.view(&mut |x| {
//!   assert_eq!(x.users[0].scores[1], 25);
//!   Ok(())
//! }).expect("Failed to read the score");
//!
//! ```

mod bridges;
mod compute;
mod error;
mod ext;
mod impls;
mod map;
mod view;

pub use error::ViewError;
pub use ext::{UnsafeViewExt, UnsafeViewMutExt, ViewExt, ViewMutExt};
pub use impls::{ArcRwLockView, ArcView};
pub use view::{UnsafeView, UnsafeViewMut, View, ViewMut};
