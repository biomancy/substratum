use crate::memory::pool::{IntoPool, PoolLease};
use std::fmt::Debug;
use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::ops::{Deref, DerefMut};

/// A temporary, exclusive loan of a pooled object.
///
/// A `Lease` is a "must-use" wrapper that pairs a pooled object (`Obj`) with
/// implementation-defined metadata (`Tag`). Its primary purpose is to ensure that a
/// loaned object's lifecycle is explicitly managed, preventing its underlying memory
/// allocation from being accidentally released.
///
/// To enforce this, the `Lease` will panic if it is dropped. This behavior
/// only occurs in debug builds to aid development; in release builds, the object's
/// memory will be silently released to avoid a performance penalty.
///
/// ### Lifecycle
/// A `Lease` must be consumed in one of two ways:
/// 1.  **Return to the pool:** Pass the lease to a [`PoolRelease::reclaim`] method. This is the
///     most common and efficient path, as it may use the lease's `Tag` to
///     optimize the release operation.
/// 2.  **Take ownership:** Call a method like [`.take()`](Lease::take) to consume the lease and gain full
///     ownership of the object. You are now responsible for its final disposition.
#[must_use = "Lease will panic on drop in debug builds; it must be explicitly consumed (e.g., via a PoolReleaser or by calling .take())"]
#[derive(Debug)]
pub struct TaggedLease<'a, Tag, Obj> {
    // Implementation-defined metadata used to optimize release operations.
    tag: Option<Tag>,
    // The underlying pooled object.
    obj: Option<Obj>,
    // Binds the lease to the lifetime of the Acquirer that created it.
    phantom: PhantomData<&'a ()>,
}

impl<'a, Tag, Obj> TaggedLease<'a, Tag, Obj> {
    /// Creates a new `TaggedLease` containing the object and its associated metadata.
    pub fn new(tag: Tag, obj: Obj) -> Self {
        Self {
            tag: Some(tag),
            obj: Some(obj),
            phantom: PhantomData,
        }
    }

    /// Returns a reference to the metadata tag associated with this lease.
    pub fn tag(&self) -> &Tag {
        // SAFETY: The `tag` field is guaranteed to be `Some` for the entire
        // lifetime of the `TaggedLease`.
        unsafe { self.tag.as_ref().unwrap_unchecked() }
    }

    /// Consumes the lease, returning the object and its metadata tag as a tuple.
    ///
    /// This is the primary method for completely consuming the lease,
    /// bypassing the panicking `Drop` implementation.
    #[must_use]
    pub fn into_parts(self) -> (Tag, Obj) {
        // Prevent our custom Drop from running, as we are moving the contents out.
        let mut me = ManuallyDrop::new(self);
        // SAFETY: `tag` and `obj` are guaranteed to be `Some` because this method
        // consumes `self`, preventing any further access.
        let (tag, obj, _) = unsafe {
            (
                me.tag.take().unwrap_unchecked(),
                me.obj.take().unwrap_unchecked(),
                me.phantom,
            )
        };
        (tag, obj)
    }
}

impl<'a, Tag, Obj> PoolLease<'a, Obj> for TaggedLease<'a, Tag, Obj>
where
    Obj: IntoPool + 'a,
{
    /// Consumes the lease, returning the underlying pooled object.
    ///
    /// After calling this method, you gain full ownership of the object and are
    /// responsible for its eventual release. Any metadata associated with the
    /// lease is discarded, forgoing potential release-path optimizations.
    fn take(self) -> Obj {
        self.into_parts().1
    }
}

impl<Tag, Obj> Deref for TaggedLease<'_, Tag, Obj> {
    type Target = Obj;

    fn deref(&self) -> &Self::Target {
        // SAFETY: The `obj` field is guaranteed to be `Some` for the entire
        // lifetime of the lease, as it is only taken during consumption.
        unsafe { self.obj.as_ref().unwrap_unchecked() }
    }
}

impl<Tag, Obj> DerefMut for TaggedLease<'_, Tag, Obj> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: The `obj` field is guaranteed to be `Some` for the entire
        // lifetime of the lease, as it is only taken during consumption.
        unsafe { self.obj.as_mut().unwrap_unchecked() }
    }
}

impl<Tag, Obj> Drop for TaggedLease<'_, Tag, Obj> {
    fn drop(&mut self) {
        debug_assert!(
            false,
            "A pooled Lease was dropped. \
            To prevent memory release, it must be consumed by returning it to a pool \
            Releaser or by taking ownership with .take() or .into_parts()."
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tagged_lease() {
        let mut lease = TaggedLease::new("tag", 42);
        assert_eq!(lease.tag(), &"tag");
        assert_eq!(*lease, 42);

        *lease = 345;
        assert_eq!(lease.into_parts(), ("tag", 345));
    }

    #[test]
    fn test_tagged_lease_take() {
        let lease = TaggedLease::new("tag", 42);
        let obj = lease.take();
        assert_eq!(obj, 42);
    }

    #[test]
    #[should_panic]
    #[cfg(debug_assertions)]
    fn test_tagged_lease_drop() {
        let lease = TaggedLease::new("tag", 42);
        // The lease is dropped here, which should panic in debug builds.
        drop(lease);
    }
}
