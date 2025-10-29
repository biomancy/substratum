use super::bridges::{Freeze, FreezeUnsafe, SafeAsUnsafe, SafeAsUnsafeMut};
use super::compute::{ComputeUnsafeView, ComputeUnsafeViewMut, ComputeView, ComputeViewMut};
use super::map::{
    MapUnsafeView, MapUnsafeViewMut, MapView, MapViewMut, TryMapUnsafeView, TryMapUnsafeViewMut,
    TryMapView,
};
use super::view::{UnsafeView, UnsafeViewMut, View, ViewMut};
use std::sync::Arc;

/// An extension trait for [`View`] providing combinator methods.
///
/// This trait is blanket-implemented for all types that implement [`View`]
/// and provides a fluent, chainable API for creating more complex views
/// from simpler ones.
pub trait ViewExt: View {
    /// Creates a new view that computes a new value from the original.
    ///
    /// This is a "reference-to-value" transformation. The provided closure `f`
    /// receives a reference to the original view's value and is responsible for
    /// computing a new value `U`. It then calls the provided callback
    /// with a reference to that new value.
    ///
    /// This is useful for "views" of computed data that isn't directly
    /// referenceable from the source, such as `string.len()` or for
    /// views that require locking or other forms of access control.
    fn compute<F, U>(self, f: F) -> ComputeView<Self, F, U>
    where
        Self: Sized,
        F: Fn(
            &Self::Value,
            &mut dyn FnMut(&U) -> Result<(), Self::Error>,
        ) -> Result<(), Self::Error>,
    {
        ComputeView::new(self, f)
    }

    /// Creates a new view that maps an immutable reference to its sub-part.
    ///
    /// This is the primary "reference-to-reference" transformation, used
    /// for navigating into nested data structures. The provided closure `f`
    /// takes a reference to the original value and returns a sub-reference
    /// (e.g., to a field or an array element).
    fn map<F, U>(self, f: F) -> MapView<Self, F, U>
    where
        Self: Sized,
        F: Fn(&Self::Value) -> &U,
    {
        MapView::new(self, f)
    }

    /// Creates a new view that fallibly maps an immutable reference to its sub-part.
    ///
    /// This is similar to `map`, but the provided closure `f` can return a [`Result`].
    fn try_map<F, E, U>(self, f: F) -> TryMapView<Self, F, E, U>
    where
        Self: Sized,
        F: Fn(&Self::Value) -> Result<&U, E>,
        Self::Error: From<E>,
    {
        TryMapView::new(self, f)
    }

    /// Turn the view into an [`UnsafeView`].
    ///
    /// This transformation should be used with care, as it strips away
    /// Rust's safety guarantees for all subsequent operations on the view.
    fn into_unsafe(self) -> SafeAsUnsafe<Self>
    where
        Self: Sized,
    {
        SafeAsUnsafe::new(self)
    }

    /// Consumes this [`View`] and returns it as a type-erased, thread-safe [`Arc`].
    fn arced(self) -> Arc<dyn View<Value = Self::Value, Error = Self::Error> + Send + Sync>
    where
        Self: 'static + Send + Sync + Sized,
    {
        Arc::new(self)
    }

    /// Consumes this [`View`] and returns it as a type-erased [`Box`].
    fn boxed(self) -> Box<dyn View<Value = Self::Value, Error = Self::Error>>
    where
        Self: 'static + Sized,
    {
        Box::new(self)
    }
}

impl<T: View> ViewExt for T {}

/// An extension trait for [`ViewMut`] providing mutable combinator methods.
///
/// This trait is blanket-implemented for all types that implement [`ViewMut`]
/// and provides a fluent, chainable API for creating more complex mutable
/// views.
pub trait ViewMutExt: ViewMut {
    /// Creates a new mutable view that computes a new value.
    ///
    /// This is the mutable-compatible version of [`ViewExt::compute`].
    /// The closure `f` receives a *mutable* reference to the original value,
    /// allowing it to perform modifications before (or after) computing the
    /// new value `U` and invoking the callback.
    fn compute_mut<F, U>(self, f: F) -> ComputeViewMut<Self, F, U>
    where
        Self: Sized,
        F: Fn(
            &mut Self::Value,
            &mut dyn FnMut(&mut U) -> Result<(), Self::Error>,
        ) -> Result<(), Self::Error>,
    {
        ComputeViewMut::new(self, f)
    }

    /// Creates a new view that maps a mutable reference to its sub-part.
    ///
    /// This is the mutable version of [`ViewExt::map`].
    fn map_mut<F, U>(self, f: F) -> MapViewMut<Self, F, U>
    where
        Self: Sized,
        F: Fn(&mut Self::Value) -> &mut U,
    {
        MapViewMut::new(self, f)
    }

    /// Creates a new view that maps a mutable reference to its sub-part.
    ///
    /// This is the mutable version of [`ViewExt::map`].
    fn try_map_mut<F, E, U>(self, f: F) -> TryMapView<Self, F, E, U>
    where
        Self: Sized,
        F: Fn(&mut Self::Value) -> Result<&mut U, E>,
        Self::Error: From<E>,
    {
        TryMapView::new(self, f)
    }

    /// Turn the view into an [`UnsafeViewMut`].
    ///
    /// This transformation should be used with care, as it strips away
    /// Rust's safety guarantees for all subsequent operations on the view.
    fn into_unsafe(self) -> SafeAsUnsafeMut<Self>
    where
        Self: Sized,
    {
        SafeAsUnsafeMut::new(self)
    }

    /// Transforms this mutable view into an immutable one.
    ///
    /// Note, this operation must be used very carefully, as most
    /// mutable views rely on exclusive access guarantees that
    /// are excessive for immutable access.
    fn freeze(self) -> Freeze<Self>
    where
        Self: Sized,
    {
        Freeze::new(self)
    }

    /// Consumes this [`ViewMut`] and returns it as a type-erased, thread-safe [`Arc`].
    fn arced_mut(self) -> Arc<dyn ViewMut<Value = Self::Value, Error = Self::Error> + Send + Sync>
    where
        Self: 'static + Send + Sync + Sized,
    {
        Arc::new(self)
    }

    /// Consumes this [`ViewMut`] and returns it as a type-erased [`Box`].
    fn boxed_mut(self) -> Box<dyn ViewMut<Value = Self::Value, Error = Self::Error>>
    where
        Self: 'static + Sized,
    {
        Box::new(self)
    }
}

impl<T: ViewMut> ViewMutExt for T {}

/// An extension trait for [`UnsafeView`] providing combinator methods.
pub trait UnsafeViewExt: UnsafeView {
    /// Creates a new unsafe view that computes a new value.
    ///
    /// The closure `f` receives an immutable pointer to the original value
    /// and is responsible for computing a new value `U` and invoking
    /// the callback with a pointer to it.
    fn compute<F, U>(self, f: F) -> ComputeUnsafeView<Self, F, U>
    where
        Self: Sized,
        F: Fn(
            *const Self::Value,
            &mut dyn FnMut(*const U) -> Result<(), Self::Error>,
        ) -> Result<(), Self::Error>,
    {
        ComputeUnsafeView::new(self, f)
    }

    /// Creates a new unsafe view that maps an immutable pointer to its sub-part.
    ///
    /// The closure `f` takes an immutable pointer and returns a new
    /// immutable pointer.
    fn map<F, U>(self, f: F) -> MapUnsafeView<Self, F, U>
    where
        Self: Sized,
        F: Fn(*const Self::Value) -> *const U,
    {
        MapUnsafeView::new(self, f)
    }

    /// Creates a new unsafe view that fallibly maps an immutable pointer to its sub-part.
    ///
    /// Similar to [`Self::map`], but the provided closure `f` can return a [`Result`].
    fn try_map<F, E, U>(self, f: F) -> TryMapUnsafeView<Self, F, E, U>
    where
        Self: Sized,
        F: Fn(*const Self::Value) -> Result<*const U, E>,
        Self::Error: From<E>,
    {
        TryMapUnsafeView::new(self, f)
    }

    /// Consumes this [`UnsafeView`] and returns it as a type-erased, thread-safe [`Arc`].
    fn arced(self) -> Arc<dyn UnsafeView<Value = Self::Value, Error = Self::Error> + Send + Sync>
    where
        Self: 'static + Send + Sync + Sized,
    {
        Arc::new(self)
    }

    /// Consumes this [`UnsafeView`] and returns it as a type-erased [`Box`].
    fn boxed(self) -> Box<dyn UnsafeView<Value = Self::Value, Error = Self::Error>>
    where
        Self: 'static + Sized,
    {
        Box::new(self)
    }
}

impl<T: UnsafeView> UnsafeViewExt for T {}

/// An extension trait for [`UnsafeViewMut`] providing mutable combinator methods.
pub trait UnsafeViewMutExt: UnsafeViewMut {
    /// Creates a new unsafe mutable view that computes a new value.
    ///
    /// The closure `f` receives a mutable pointer to the original value
    /// and is responsible for computing a new value `U` and invoking
    /// the callback with a pointer to it.
    fn compute_mut<F, U>(self, f: F) -> ComputeUnsafeViewMut<Self, F, U>
    where
        Self: Sized,
        F: Fn(
            *mut Self::Value,
            &mut dyn FnMut(*mut U) -> Result<(), Self::Error>,
        ) -> Result<(), Self::Error>,
    {
        ComputeUnsafeViewMut::new(self, f)
    }

    /// Creates a new unsafe view that maps a mutable pointer to its sub-part.
    fn map_mut<F, U>(self, f: F) -> MapUnsafeViewMut<Self, F, U>
    where
        Self: Sized,
        F: Fn(*mut Self::Value) -> *mut U,
    {
        MapUnsafeViewMut::new(self, f)
    }

    /// Creates a new unsafe view that fallibly maps a mutable pointer to its sub-part.
    ///
    /// Similar to [`Self::map_mut`], but the provided closure `f` can return a [`Result`].
    fn try_map_mut<F, E, U>(self, f: F) -> TryMapUnsafeViewMut<Self, F, E, U>
    where
        Self: Sized,
        F: Fn(*mut Self::Value) -> Result<*mut U, E>,
        Self::Error: From<E>,
    {
        TryMapUnsafeViewMut::new(self, f)
    }

    /// Transforms this mutable-pointer view into an immutable-pointer one.
    fn freeze(self) -> FreezeUnsafe<Self>
    where
        Self: Sized,
    {
        FreezeUnsafe::new(self)
    }

    /// Consumes this [`UnsafeViewMut`] and returns it as a type-erased, thread-safe [`Arc`].
    fn arced_mut(
        self,
    ) -> Arc<dyn UnsafeViewMut<Value = Self::Value, Error = Self::Error> + Send + Sync>
    where
        Self: 'static + Send + Sync + Sized,
    {
        Arc::new(self)
    }

    /// Consumes this [`UnsafeViewMut`] and returns it as a type-erased [`Box`].
    fn boxed_mut(self) -> Box<dyn UnsafeViewMut<Value = Self::Value, Error = Self::Error>>
    where
        Self: 'static + Sized,
    {
        Box::new(self)
    }
}

impl<T: UnsafeViewMut> UnsafeViewMutExt for T {}
