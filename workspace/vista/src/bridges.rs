use super::view::{
    UnsafeView, UnsafeViewFn, UnsafeViewMut, UnsafeViewMutFn, View, ViewBase, ViewFn, ViewMut,
};

pub struct Freeze<V> {
    view_mut: V,
}

impl<V> Freeze<V> {
    pub fn new(view_mut: V) -> Self {
        Self { view_mut }
    }
}

impl<V: ViewMut> ViewBase for Freeze<V> {
    type Value = V::Value;
}

impl<V: ViewMut> View for Freeze<V> {
    type Error = V::Error;

    fn view(&self, f: ViewFn<'_, Self::Value, Self::Error>) -> Result<(), Self::Error> {
        self.view_mut.view_mut(&mut |x| f(x))
    }
}

/// A wrapper struct that implements [`UnsafeView`] for any [`View`].
///
/// This is an explicit bridge to allow safe, synchronized views
/// to be used in code that expects an [`UnsafeView`].
pub struct SafeAsUnsafe<V> {
    safe_view: V,
}

impl<V> SafeAsUnsafe<V> {
    pub fn new(safe_view: V) -> Self {
        Self { safe_view }
    }
}

impl<V: View> ViewBase for SafeAsUnsafe<V> {
    type Value = V::Value;
}

impl<V: View> UnsafeView for SafeAsUnsafe<V> {
    type Error = V::Error;

    /// # Safety
    /// This is trivially safe, as it calls the underlying
    /// *safe* [`View::view`] method, which performs all required locks.
    #[inline]
    fn view(&self, f: UnsafeViewFn<'_, Self::Value, Self::Error>) -> Result<(), Self::Error> {
        self.safe_view
            .view(&mut |val: &Self::Value| f(val as *const _))
    }
}

/// A wrapper struct that implements [`UnsafeViewMut`] for any [`ViewMut`].
///
/// This is an explicit bridge to allow safe, synchronized mutable views
/// to be used in code that expects an [`UnsafeViewMut`].
pub struct SafeAsUnsafeMut<V> {
    safe_view: V,
}

impl<V> SafeAsUnsafeMut<V> {
    pub fn new(safe_view: V) -> Self {
        Self { safe_view }
    }
}

impl<V: ViewMut> ViewBase for SafeAsUnsafeMut<V> {
    type Value = V::Value;
}

impl<V: ViewMut> UnsafeViewMut for SafeAsUnsafeMut<V> {
    type Error = V::Error;

    /// # Safety
    /// This is trivially safe, as it calls the underlying
    /// *safe* [`ViewMut::view_mut`] method, which performs all required locks.
    #[inline]
    fn view_mut(
        &self,
        f: UnsafeViewMutFn<'_, Self::Value, Self::Error>,
    ) -> Result<(), Self::Error> {
        self.safe_view
            .view_mut(&mut |val: &mut Self::Value| f(val as *mut _))
    }
}

/// An unsafe bridge from a mutable-pointer view to an immutable-pointer view.
pub struct FreezeUnsafe<V> {
    view_mut: V,
}

impl<V> FreezeUnsafe<V> {
    pub fn new(view_mut: V) -> Self {
        Self { view_mut }
    }
}

impl<V: UnsafeViewMut> ViewBase for FreezeUnsafe<V> {
    type Value = V::Value;
}

impl<V: UnsafeViewMut> UnsafeView for FreezeUnsafe<V> {
    type Error = V::Error;

    /// # Safety
    ///
    /// The safety requirements of [`UnsafeView::view`] must be upheld.
    /// This implementation calls [`UnsafeViewMut::view_mut`] and casts
    /// the `*mut T` to a `*const T`.
    fn view(&self, f: UnsafeViewFn<'_, Self::Value, Self::Error>) -> Result<(), Self::Error> {
        // Casting *mut T to *const T is safe.
        self.view_mut.view_mut(&mut |x| f(x as *const _))
    }
}
