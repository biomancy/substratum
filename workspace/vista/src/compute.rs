use super::view::{
    UnsafeView, UnsafeViewFn, UnsafeViewMut, UnsafeViewMutFn, View, ViewBase, ViewFn, ViewMut,
    ViewMutFn,
};

pub struct ComputeView<V, F, U> {
    view: V,
    f: F,
    _marker: std::marker::PhantomData<U>,
}

impl<V, F, U> ComputeView<V, F, U> {
    pub fn new(view: V, f: F) -> Self {
        Self {
            view,
            f,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<V, F, U> ViewBase for ComputeView<V, F, U> {
    type Value = U;
}

impl<V, F, U> View for ComputeView<V, F, U>
where
    V: View,
    F: Fn(&V::Value, &mut dyn FnMut(&U) -> Result<(), V::Error>) -> Result<(), V::Error>,
{
    type Error = V::Error;

    fn view(&self, f: ViewFn<'_, Self::Value, Self::Error>) -> Result<(), Self::Error> {
        self.view
            .view(&mut |original: &V::Value| (self.f)(original, f))
    }
}

pub struct ComputeViewMut<V, F, U> {
    view: V,
    f: F,
    _marker: std::marker::PhantomData<U>,
}

impl<V, F, U> ComputeViewMut<V, F, U> {
    pub fn new(view: V, f: F) -> Self {
        Self {
            view,
            f,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<V, F, U> ViewBase for ComputeViewMut<V, F, U> {
    type Value = U;
}

impl<V, F, U> ViewMut for ComputeViewMut<V, F, U>
where
    V: ViewMut,
    F: Fn(&mut V::Value, &mut dyn FnMut(&mut U) -> Result<(), V::Error>) -> Result<(), V::Error>,
{
    type Error = V::Error;

    fn view_mut(&self, f: ViewMutFn<'_, Self::Value, Self::Error>) -> Result<(), Self::Error> {
        self.view
            .view_mut(&mut |original: &mut V::Value| (self.f)(original, f))
    }
}

pub struct ComputeUnsafeView<V, F, U> {
    view: V,
    f: F,
    _marker: std::marker::PhantomData<U>,
}

impl<V, F, U> ComputeUnsafeView<V, F, U> {
    pub fn new(view: V, f: F) -> Self {
        Self {
            view,
            f,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<V, F, U> ViewBase for ComputeUnsafeView<V, F, U> {
    type Value = U;
}

impl<V, F, U> UnsafeView for ComputeUnsafeView<V, F, U>
where
    V: UnsafeView,
    F: Fn(
        *const V::Value,
        &mut dyn FnMut(*const U) -> Result<(), V::Error>,
    ) -> Result<(), V::Error>,
{
    type Error = V::Error;

    fn view(&self, f: UnsafeViewFn<'_, Self::Value, Self::Error>) -> Result<(), Self::Error> {
        self.view
            .view(&mut |original: *const V::Value| (self.f)(original, f))
    }
}

pub struct ComputeUnsafeViewMut<V, F, U> {
    view: V,
    f: F,
    _marker: std::marker::PhantomData<U>,
}

impl<V, F, U> ComputeUnsafeViewMut<V, F, U> {
    pub fn new(view: V, f: F) -> Self {
        Self {
            view,
            f,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<V, F, U> ViewBase for ComputeUnsafeViewMut<V, F, U> {
    type Value = U;
}

impl<V, F, U> UnsafeViewMut for ComputeUnsafeViewMut<V, F, U>
where
    V: UnsafeViewMut,
    F: Fn(*mut V::Value, &mut dyn FnMut(*mut U) -> Result<(), V::Error>) -> Result<(), V::Error>,
{
    type Error = V::Error;

    fn view_mut(
        &self,
        f: UnsafeViewMutFn<'_, Self::Value, Self::Error>,
    ) -> Result<(), Self::Error> {
        self.view
            .view_mut(&mut |original: *mut V::Value| (self.f)(original, f))
    }
}
