use crate::memory::pool::{PoolBound, PoolCore};
use crate::memory::Pool;
use std::cell::RefCell;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use thread_local::ThreadLocal;
use crate::memory::pool::lifecycle::{FromPool, IntoPool, TryFromPool};

pub struct ThreadLocalPool<PC: PoolCore> {
    factory: fn() -> PC,
    pool: Arc<ThreadLocal<RefCell<PC>>>,
}

// impl<PC: PoolCore> ThreadLocalPool<PC> {
//     pub fn from_factory(factory: fn() -> PC) -> Self {
//         ThreadLocalPool {
//             factory,
//             pool: Arc::new(ThreadLocal::new()),
//         }
//     }
// }
// 
// impl<PC: PoolCore> Clone for ThreadLocalPool<PC> {
//     fn clone(&self) -> Self {
//         ThreadLocalPool {
//             factory: self.factory,
//             pool: self.pool.clone(),
//         }
//     }
// }
// 
// pub struct ThreadLocalGuard<'a, PC: PoolCore, T: IntoPool> {
//     obj: Option<T>,
//     pool: &'a RefCell<PC>,
// }
// 
// impl<PC: PoolCore, T: IntoPool> Deref for ThreadLocalGuard<'_, PC, T> {
//     type Target = T;
// 
//     fn deref(&self) -> &Self::Target {
//         unsafe { self.obj.as_ref().unwrap_unchecked() }
//     }
// }
// 
// impl<PC: PoolCore, T: IntoPool> DerefMut for ThreadLocalGuard<'_, PC, T> {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         unsafe { self.obj.as_mut().unwrap_unchecked() }
//     }
// }
// 
// impl<PC: PoolCore, T: IntoPool> Drop for ThreadLocalGuard<'_, PC, T> {
//     fn drop(&mut self) {
//         if let Some(obj) = self.obj.take() {
//             // SAFETY: Pools is guaranteed to be initialized for the current thread.
//             // Guard cannot be shared across threads, so we can safely unwrap.
//             self.pool.borrow_mut().release(obj);
//         }
//     }
// }
// 
// impl<'a, PC: PoolCore, T: IntoPool + 'a> Guard<'a, T> for ThreadLocalGuard<'a, PC, T> {
//     fn take(mut self) -> T {
//         let obj = unsafe { self.obj.take().unwrap_unchecked() };
//         obj
//     }
// }
// 
// impl<PC: PoolCore> Pool for ThreadLocalPool<PC> {
//     type Guard<'a, T: 'a + IntoPool> = ThreadLocalGuard<'a, PC, T>;
// 
//     fn acquire<'a, T: FromPool + IntoPool + 'a>(&'a self) -> Self::Guard<'a, T> {
//         let pool = self.pool.get_or(|| RefCell::new((self.factory)()));
//         let obj = pool.borrow_mut().acquire();
//         ThreadLocalGuard {
//             obj: Some(obj),
//             pool,
//         }
//     }
// 
//     fn try_acquire<'a, T: TryFromPool + IntoPool + 'a>(&'a self) -> Option<Self::Guard<'a, T>> {
//         let pool = self.pool.get_or(|| RefCell::new((self.factory)()));
//         pool.borrow_mut()
//             .try_acquire::<T>()
//             .map(|obj| ThreadLocalGuard {
//                 obj: Some(obj),
//                 pool,
//             })
//     }
// 
//     fn release<T: IntoPool>(&self, obj: T) {
//         // Objects could be send across threads, but the pool itself is thread-local.
//         // Therefore, we can't assume the pool is initialized for the current thread.
//         self.pool
//             .get_or(|| RefCell::new((self.factory)()))
//             .borrow_mut()
//             .release(obj);
//     }
// 
//     fn clear(&self) {
//         if let Some(pool) = self.pool.get() {
//             pool.borrow_mut().clear()
//         }
//     }
// 
//     fn optimize(&self) {
//         if let Some(pool) = self.pool.get() {
//             pool.borrow_mut().optimize()
//         }
//     }
// }
