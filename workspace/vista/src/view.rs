/// Base trait for types that provide view access to an underlying value.
pub trait ViewBase {
    type Value;
}

/// A type alias for a mutable closure that takes an immutable reference
/// to a value of type `V` and returns a `Result`.
pub type ViewFn<'scope, V, E> = &'scope mut dyn for<'a> FnMut(&'a V) -> Result<(), E>;

/// Provides immutable, closure-based access to a value.
///
/// The [`View`] trait implements a "lending" or "callback" pattern to safely
/// grant temporary access to a value of type [`View::Value`]. This pattern is essential
/// for abstracting over data access strategies where a direct reference
/// cannot be simply returned, such as:
///
/// * Data protected by interior mutability (e.g., [`std::sync::RwLock`], [`std::sync::Mutex`]).
/// * Data that is generated or calculated on-the-fly.
/// * Data accessed through a complex, fallible navigation path (e.g., nested
///   structures or collections).
///
/// By forcing all access to occur *within* a provided closure (`f`),
/// implementors can precisely control the lifetime of the borrow. The reference
/// to [`View::Value`] is guaranteed to be valid only for the duration of the
/// closure's execution. This sidesteps lifetime management problems,
/// especially when interfacing across FFI boundaries.
pub trait View: ViewBase {
    /// The type of error that can occur when attempting to access the value.
    type Error: std::error::Error;

    /// Executes a closure with an immutable reference to the underlying value.
    ///
    /// The implementation is responsible for "materializing" the reference to
    /// [`Self::Value`] (e.g., by acquiring a lock or generating the data)
    /// and then invoking the closure `f` with it.
    ///
    /// # Parameters
    ///
    /// * `f`: A mutable closure that will be called with an immutable reference
    ///   to [`Self::Value`].
    ///
    /// # Errors
    ///
    /// This function propagates errors from two sources:
    ///
    /// 1. If the view itself fails to materialize the value (e.g., a lock
    ///    is poisoned, an index is out of bounds, etc.).
    /// 2. If the provided closure `f` returns an [`Err`].
    fn view(&self, f: ViewFn<'_, Self::Value, Self::Error>) -> Result<(), Self::Error>;
}

/// A type alias for a mutable closure that takes an immutable reference
/// to a value of type `V` and returns a `Result`.
pub type ViewMutFn<'scope, V, E> = &'scope mut dyn for<'a> FnMut(&'a mut V) -> Result<(), E>;

/// Provides mutable, closure-based access to a value.
///
/// This trait extends `View` to allow for *mutable* operations on the
/// underlying value. It follows the same "lending" pattern, ensuring that
/// the mutable reference is valid *only* within the scope of the
/// provided closure.
///
/// This is the standard abstraction for allowing safe, in-place modification
/// of data that is not directly accessible, such as data within an `RwLock`
/// or an element deep within a nested structure.
pub trait ViewMut: ViewBase {
    /// The type of error that can occur when attempting to access the value.
    type Error: std::error::Error;

    /// Executes a closure with a mutable reference to the underlying value.
    ///
    /// The implementation is responsible for materializing the *mutable*
    /// reference (e.g., by acquiring a write lock) and then invoking the
    /// closure `f` with it.
    ///
    /// # Parameters
    ///
    /// * `f`: A mutable closure that will be called with a *mutable* reference
    ///   to [`Self::Value`], allowing for in-place modification.
    ///
    /// # Errors
    ///
    /// This function propagates errors from two sources:
    ///
    /// 1. If the view itself fails to materialize the mutable value (e.g., a
    ///    write lock is poisoned).
    /// 2. If the provided closure `f` returns an `Err`.
    fn view_mut(&self, f: ViewMutFn<'_, Self::Value, Self::Error>) -> Result<(), Self::Error>;
}

/// A type alias for a mutable closure that takes a raw pointer
/// to a value of type `V` and returns a `Result`.
pub type UnsafeViewFn<'scope, V, E> = &'scope mut dyn FnMut(*const V) -> Result<(), E>;

/// Provides unsafe, raw pointer-based immutable access to an underlying value.
///
/// This trait is an "escape hatch" for performance-critical scenarios.
/// Callers are responsible for ensuring all synchronization and aliasing rules.
pub trait UnsafeView: ViewBase {
    /// The type of error that can occur.
    type Error: std::error::Error;

    /// Executes a closure with a raw, immutable pointer to the underlying value.
    ///
    /// # Safety
    ///
    /// The caller *must* guarantee that no other thread is writing to the
    /// data pointed to for the duration of the closure `f`. Concurrent
    /// reads are only safe if the underlying type supports them.
    /// The caller must uphold all of Rust's aliasing rules.
    fn view(&self, f: UnsafeViewFn<'_, Self::Value, Self::Error>) -> Result<(), Self::Error>;
}

/// A type alias for a mutable closure that takes a raw pointer
/// to a value of type `V` and returns a `Result`.
pub type UnsafeViewMutFn<'scope, V, E> = &'scope mut dyn FnMut(*mut V) -> Result<(), E>;

/// Provides unsafe, raw pointer-based mutable access to an underlying value.
///
/// Callers are responsible for ensuring all synchronization and aliasing rules.
pub trait UnsafeViewMut: ViewBase {
    /// The type of error that can occur.
    type Error: std::error::Error;

    /// Executes a closure with a raw, mutable pointer to the underlying value.
    ///
    /// # Safety
    ///
    /// The caller *must* guarantee that no other thread is reading *or*
    /// writing to the data pointed to for the duration of the closure `f`.
    /// The caller must uphold all of Rust's mutable aliasing rules.
    fn view_mut(&self, f: UnsafeViewMutFn<'_, Self::Value, Self::Error>)
    -> Result<(), Self::Error>;
}
