use crate::memory::{ArrayAllocation, ObjectAllocation};
use std::alloc::Layout;
use std::any::{Any, TypeId};

/// Defines how a complex object deconstructs itself to be recycled by a `Pool`.
///
/// This trait is central to the library's recursive pooling mechanism. An object
/// implementing [`IntoPool`] knows how to return its internal components—both raw
/// memory allocations and other complex [`IntoPool`] objects—back to the pool via the
/// provided handle.
///
/// When deconstructing nested objects, you can choose between using the
/// [`PoolReleaseHandle::release`] method or invoking their [`IntoPool::into_pool`] methods
/// directly (e.g., `object.into_pool(handle)`). See the [`PoolReleaseHandle::release`]
/// documentation for more details.
///
/// Types implementing this trait must specify an associated `Identity` type,
/// which serves as a logical identifier within the pool. This identity can be
/// shared across multiple related types (e.g., `MyObject` and `MyObjectBuilder`)
/// or generic type instantiations (e.g., `MyObject<'a, T>` and `MyObject<'b, U>`
/// can share `Identity = MyObject<'static, ()>`). This shared identity enables semantic
/// grouping of types, allowing for optimizations in the pool's implementation.
/// For instance, the pool can treat allocations for `MyObject` and `MyObjectBuilder`
/// similarly, facilitating recycling and avoiding unnecessary duplication.
// TODO: Consider adding a `PoolIdentity` trait for more flexible identity management once
//       specialization is stable in Rust.
pub trait IntoPool {
    type Identity: 'static + Any;
    fn into_pool(self, handle: &mut impl PoolReleaseHandle);
}

/// Defines how to **infallibly** construct an object using resources from a `Pool`.
///
/// This construction path is guaranteed to succeed, creating new default allocations
/// if no cached versions are available in the pool. See [`TryFromPool`] for a fallible version
/// of this trait.
///
/// Similarly to [`IntoPool`], nested objects can be constructed either by calling
/// the [`PoolAcquireHandle::acquire`] method directly or by invoking their [`FromPool::from_pool`] 
/// methods. See the documentation for [`PoolAcquireHandle::acquire`] for more details.
pub trait FromPool: IntoPool {
    fn from_pool(handle: &mut impl PoolAcquireHandle) -> Self;
}

/// Defines how to **fallibly** construct an object using only cached resources.
///
/// This trait provides a "best-effort" construction path. If any required component
/// is not already cached in the pool, the construction must fail and return `None`.
/// See [`FromPool`] for an infallible version of this trait.
///
/// Similarly to [`IntoPool`], nested objects can be constructed either by calling
/// the [`PoolAcquireHandle::try_acquire`] method directly or by invoking their 
/// [`TryFromPool::try_from_pool`]  methods. See the documentation for 
/// [`PoolAcquireHandle::try_acquire`] for more details.
pub trait TryFromPool: IntoPool + Sized {
    fn try_from_pool(handle: &mut impl PoolAcquireHandle) -> Option<Self>;
}

/// A handle provided during deconstruction that allows an object to return its
/// components to the pool.
pub trait PoolReleaseHandle {
    /// Returns a raw object allocation to the pool for recycling.
    /// The `identity` provides a hint about the logical nature of the object being recycled.
    fn put_object(&mut self, identity: TypeId, allocation: ObjectAllocation);

    /// Returns a raw memory array block to the pool for recycling.
    /// The `identity` provides a hint about the logical nature of the array being recycled.
    fn put_array(&mut self, identity: TypeId, allocation: ArrayAllocation);

    /// Recursively deconstructs a nested component that implements `IntoPool`.
    ///
    /// Use:
    /// * `T::into_pool(handle)` to decompose an object, treating it as an integral part of the
    ///   current object. This is equivalent to "unrolling" the nested object logic into the
    ///   current deconstruction call.
    /// * `ReleaseHandle::release<T>(nested)` to decompose an object, treating it as a separate
    ///   entity. This is logically equivalent to a separate [`PoolRelease::release`] call.
    fn release<T: IntoPool>(&mut self, obj: T);
}

/// A handle provided during construction that allows an object to acquire components
/// from the pool.
pub trait PoolAcquireHandle {
    /// Retrieves an allocation from the pool that matches the specified layout. The
    /// `identity` provides a hint about the logical nature of the object being requested.
    ///
    /// Returns `None` if no cached version is available.
    fn take_object(&mut self, identity: TypeId, layout: Layout) -> Option<ObjectAllocation>;

    /// Retrieves an allocation from the pool that matches the specified
    /// alignment and element size. The `identity` provides a hint about the logical
    /// nature of the array being requested.
    ///
    /// Returns `None` if no cached version is available.
    fn take_array(
        &mut self,
        identity: TypeId,
        alignment: usize,
        element_size: usize,
    ) -> Option<ArrayAllocation>;

    /// **Infallibly** composes a nested object.
    ///
    /// Recursively acquires all necessary sub-components, allocating new ones if
    /// the pool's core are empty.
    ///
    /// Use:
    /// * `T::from_pool(handle)` to acquire an object from the pool, treating it as an
    ///   integral part of the current object.
    /// * `AcquireHandle::acquire<T>()` to acquire an object from the pool, treating it as a
    ///   separate entity. This is logically equivalent to calling [`PoolAcquire::acquire`].
    fn acquire<T: FromPool>(&mut self) -> T;

    /// **Fallibly** composes a nested object from cached components.
    ///
    /// Returns `None` if any required sub-component is not available in the core.
    ///
    /// Use:
    /// * `T::try_from_pool(handle)` to attempt to acquire an object from the pool, treating it as an
    ///   integral part of the current object.
    /// * `AcquireHandle::try_acquire<T>()` to attempt to acquire an object
    ///   from the pool, treating it as a separate entity. This is logically equivalent to
    ///   calling [`PoolAcquire::try_acquire`].
    fn try_acquire<T: TryFromPool>(&mut self) -> Option<T>;
}
