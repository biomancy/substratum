use super::error::ViewError;
use super::view::ViewMut;
use super::view::{View, ViewBase, ViewFn, ViewMutFn};
use std::convert::Infallible;
use std::marker::PhantomData;
use std::ops::Deref;
use std::sync::{Arc, RwLock};

#[derive(Debug)]
pub struct ArcView<T, Error = Infallible> {
    pub inner: Arc<T>,
    pub _marker: PhantomData<Error>,
}

impl<T, Error> ArcView<T, Error> {
    pub fn new(inner: Arc<T>) -> Self {
        Self {
            inner,
            _marker: PhantomData,
        }
    }
}

impl<T, Error> Clone for ArcView<T, Error> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            _marker: PhantomData,
        }
    }
}

impl<T, Error> From<Arc<T>> for ArcView<T, Error> {
    fn from(value: Arc<T>) -> Self {
        Self::new(value)
    }
}

impl<T, Error> Deref for ArcView<T, Error> {
    type Target = Arc<T>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T, Error> ViewBase for ArcView<T, Error> {
    type Value = T;
}

impl<T, Error: std::error::Error> View for ArcView<T, Error> {
    type Error = Error;

    fn view(&self, f: ViewFn<'_, Self::Value, Self::Error>) -> Result<(), Self::Error> {
        f(self.inner.as_ref())
    }
}

#[derive(Debug)]
pub struct ArcRwLockView<T> {
    pub inner: Arc<RwLock<T>>,
}

impl<T> ArcRwLockView<T> {
    pub fn new(inner: Arc<RwLock<T>>) -> Self {
        Self { inner }
    }
}

impl<T> Clone for ArcRwLockView<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<T> From<Arc<RwLock<T>>> for ArcRwLockView<T> {
    fn from(value: Arc<RwLock<T>>) -> Self {
        Self::new(value)
    }
}

impl<T> Deref for ArcRwLockView<T> {
    type Target = Arc<RwLock<T>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> ViewBase for ArcRwLockView<T> {
    type Value = T;
}

impl<T> View for ArcRwLockView<T> {
    type Error = ViewError;

    fn view(&self, f: ViewFn<'_, Self::Value, Self::Error>) -> Result<(), Self::Error> {
        f(&*self.inner.read()?)
    }
}

impl<T> ViewMut for ArcRwLockView<T> {
    type Error = ViewError;

    fn view_mut(&self, f: ViewMutFn<'_, Self::Value, Self::Error>) -> Result<(), Self::Error> {
        f(&mut *self.inner.write()?)
    }
}
