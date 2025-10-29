use super::view::{
    UnsafeView, UnsafeViewFn, UnsafeViewMut, UnsafeViewMutFn, View, ViewBase, ViewFn, ViewMut,
    ViewMutFn,
};

pub struct MapView<V, F, U> {
    view: V,
    f: F,
    _marker: std::marker::PhantomData<U>,
}

impl<V, F, U> MapView<V, F, U> {
    pub fn new(view: V, f: F) -> Self {
        Self {
            view,
            f,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<V, F, U> ViewBase for MapView<V, F, U> {
    type Value = U;
}

impl<V, F, U> View for MapView<V, F, U>
where
    V: View,
    F: Fn(&V::Value) -> &U,
{
    type Error = V::Error;

    fn view(&self, f: ViewFn<'_, Self::Value, Self::Error>) -> Result<(), Self::Error> {
        self.view.view(&mut |original: &V::Value| {
            let projected: &U = (self.f)(original);
            f(projected)
        })
    }
}

pub struct TryMapView<V, F, E, U> {
    view: V,
    f: F,
    _marker: std::marker::PhantomData<(E, U)>,
}

impl<V, F, E, U> TryMapView<V, F, E, U> {
    pub fn new(view: V, f: F) -> Self {
        Self {
            view,
            f,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<V, F, E, U> ViewBase for TryMapView<V, F, E, U> {
    type Value = U;
}

impl<V, F, E, U> View for TryMapView<V, F, E, U>
where
    V: View,
    V::Error: From<E>,
    F: Fn(&V::Value) -> Result<&U, E>,
{
    type Error = V::Error;

    fn view(&self, f: ViewFn<'_, Self::Value, Self::Error>) -> Result<(), Self::Error> {
        self.view.view(&mut |original: &V::Value| {
            let projected: &U = (self.f)(original).map_err(V::Error::from)?;
            f(projected)
        })
    }
}

pub struct MapViewMut<V, F, U> {
    view: V,
    f: F,
    _marker: std::marker::PhantomData<U>,
}

impl<V, F, U> MapViewMut<V, F, U> {
    pub fn new(view: V, f: F) -> Self {
        Self {
            view,
            f,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<V, F, U> ViewBase for MapViewMut<V, F, U> {
    type Value = U;
}

impl<V, F, U> ViewMut for MapViewMut<V, F, U>
where
    V: ViewMut,
    F: Fn(&mut V::Value) -> &mut U,
{
    type Error = V::Error;

    fn view_mut(&self, f: ViewMutFn<'_, Self::Value, Self::Error>) -> Result<(), Self::Error> {
        self.view.view_mut(&mut |original: &mut V::Value| {
            let projected: &mut U = (self.f)(original);
            f(projected)
        })
    }
}

impl<V, F, E, U> ViewMut for TryMapView<V, F, E, U>
where
    V: ViewMut,
    V::Error: From<E>,
    F: Fn(&mut V::Value) -> Result<&mut U, E>,
{
    type Error = V::Error;

    fn view_mut(&self, f: ViewMutFn<'_, Self::Value, Self::Error>) -> Result<(), Self::Error> {
        self.view.view_mut(&mut |original: &mut V::Value| {
            let projected: &mut U = (self.f)(original).map_err(V::Error::from)?;
            f(projected)
        })
    }
}

pub struct MapUnsafeView<V, F, U> {
    view: V,
    f: F,
    _marker: std::marker::PhantomData<U>,
}

impl<V, F, U> MapUnsafeView<V, F, U> {
    pub fn new(view: V, f: F) -> Self {
        Self {
            view,
            f,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<V, F, U> ViewBase for MapUnsafeView<V, F, U> {
    type Value = U;
}

impl<V, F, U> UnsafeView for MapUnsafeView<V, F, U>
where
    V: UnsafeView,
    F: Fn(*const V::Value) -> *const U,
{
    type Error = V::Error;

    fn view(&self, f: UnsafeViewFn<'_, Self::Value, Self::Error>) -> Result<(), Self::Error> {
        self.view.view(&mut |original: *const V::Value| {
            let projected: *const U = (self.f)(original);
            f(projected)
        })
    }
}

pub struct TryMapUnsafeView<V, F, E, U> {
    view: V,
    f: F,
    _marker: std::marker::PhantomData<(E, U)>,
}

impl<V, F, E, U> TryMapUnsafeView<V, F, E, U> {
    pub fn new(view: V, f: F) -> Self {
        Self {
            view,
            f,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<V, F, E, U> ViewBase for TryMapUnsafeView<V, F, E, U> {
    type Value = U;
}

impl<V, F, E, U> UnsafeView for TryMapUnsafeView<V, F, E, U>
where
    V: UnsafeView,
    V::Error: From<E>,
    F: Fn(*const V::Value) -> Result<*const U, E>,
{
    type Error = V::Error;

    fn view(&self, f: UnsafeViewFn<'_, Self::Value, Self::Error>) -> Result<(), Self::Error> {
        self.view.view(&mut |original: *const V::Value| {
            let projected: *const U = (self.f)(original).map_err(V::Error::from)?;
            f(projected)
        })
    }
}

pub struct MapUnsafeViewMut<V, F, U> {
    view: V,
    f: F,
    _marker: std::marker::PhantomData<U>,
}

impl<V, F, U> MapUnsafeViewMut<V, F, U> {
    pub fn new(view: V, f: F) -> Self {
        Self {
            view,
            f,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<V, F, U> ViewBase for MapUnsafeViewMut<V, F, U> {
    type Value = U;
}

impl<V, F, U> UnsafeViewMut for MapUnsafeViewMut<V, F, U>
where
    V: UnsafeViewMut,
    F: Fn(*mut V::Value) -> *mut U,
{
    type Error = V::Error;

    fn view_mut(
        &self,
        f: UnsafeViewMutFn<'_, Self::Value, Self::Error>,
    ) -> Result<(), Self::Error> {
        self.view.view_mut(&mut |original: *mut V::Value| {
            let projected: *mut U = (self.f)(original);
            f(projected)
        })
    }
}

pub struct TryMapUnsafeViewMut<V, F, E, U> {
    view: V,
    f: F,
    _marker: std::marker::PhantomData<(E, U)>,
}

impl<V, F, E, U> TryMapUnsafeViewMut<V, F, E, U> {
    pub fn new(view: V, f: F) -> Self {
        Self {
            view,
            f,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<V, F, E, U> ViewBase for TryMapUnsafeViewMut<V, F, E, U> {
    type Value = U;
}

impl<V, F, E, U> UnsafeViewMut for TryMapUnsafeViewMut<V, F, E, U>
where
    V: UnsafeViewMut,
    V::Error: From<E>,
    F: Fn(*mut V::Value) -> Result<*mut U, E>,
{
    type Error = V::Error;

    fn view_mut(
        &self,
        f: UnsafeViewMutFn<'_, Self::Value, Self::Error>,
    ) -> Result<(), Self::Error> {
        self.view.view_mut(&mut |original: *mut V::Value| {
            let projected: *mut U = (self.f)(original).map_err(V::Error::from)?;
            f(projected)
        })
    }
}
