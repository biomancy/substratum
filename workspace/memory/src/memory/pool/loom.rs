// use crate::memory::pool::core::core::{AffinityArrayCache, FlatObjectCache};
// use crate::memory::pool::core::core::PoolCore;
// use crate::memory::pool::lifecycle::{FromPool, IntoPool, PoolAcquireHandle, PoolReleaseHandle, TryFromPool};
// use crate::memory::{ArrayAllocation, ObjectAllocation};
// use std::alloc::Layout;
// use std::any::TypeId;
//
// pub struct StealingAffinityPool {
//     objects: FlatObjectCache,
//     arrays: AffinityArrayCache,
// }
//
// impl Default for StealingAffinityPool {
//     fn default() -> Self {
//         StealingAffinityPool {
//             objects: FlatObjectCache::default(),
//             arrays: AffinityArrayCache::default(),
//         }
//     }
// }
//
// struct ReleaseCtx<'a> {
//     pool: &'a mut StealingAffinityPool,
//     top_identity: TypeId,
// }
//
// impl PoolReleaseHandle for ReleaseCtx<'_> {
//     #[inline]
//     fn put_object(&mut self, _: TypeId, allocation: ObjectAllocation) {
//         self.pool.objects.put(allocation)
//     }
//
//     #[inline]
//     fn put_array(&mut self, _: TypeId, allocation: ArrayAllocation) {
//         self.pool.arrays.put(
//             &self.top_identity,
//             allocation.layout().align(),
//             allocation.element_size(),
//             allocation,
//         )
//     }
//
//     #[inline]
//     fn release<T: IntoPool>(&mut self, obj: T) {
//         let previous_identity = self.top_identity;
//         self.top_identity = TypeId::of::<T::Identity>();
//         obj.into_pool(self);
//         self.top_identity = previous_identity;
//     }
// }
//
// struct AcquireCtx<'a> {
//     pool: &'a mut StealingAffinityPool,
//     top_identity: TypeId,
// }
//
// impl PoolAcquireHandle for AcquireCtx<'_> {
//     #[inline]
//     fn take_object(&mut self, _: TypeId, layout: Layout) -> Option<ObjectAllocation> {
//         self.pool.objects.take(&layout)
//     }
//
//     #[inline]
//     fn take_array(
//         &mut self,
//         _: TypeId,
//         alignment: usize,
//         element_size: usize,
//     ) -> Option<ArrayAllocation> {
//         self.pool
//             .arrays
//             .take(&self.top_identity, alignment, element_size)
//     }
//
//     #[inline]
//     fn acquire<T: FromPool>(&mut self) -> T {
//         let previous_identity = self.top_identity;
//         self.top_identity = TypeId::of::<T::Identity>();
//         let obj = T::from_pool(self);
//         self.top_identity = previous_identity;
//         obj
//     }
//
//     #[inline]
//     fn try_acquire<T: TryFromPool>(&mut self) -> Option<T> {
//         let previous_identity = self.top_identity;
//         self.top_identity = TypeId::of::<T::Identity>();
//         let obj = T::try_from_pool(self);
//         self.top_identity = previous_identity;
//         obj
//     }
// }
//
// impl PoolCore for StealingAffinityPool {
//     #[inline]
//     fn acquire<T: FromPool + IntoPool>(&mut self) -> T {
//         let mut ctx = AcquireCtx {
//             pool: self,
//             top_identity: TypeId::of::<T::Identity>(),
//         };
//         T::from_pool(&mut ctx)
//     }
//
//     #[inline]
//     fn try_acquire<T: TryFromPool + IntoPool>(&mut self) -> Option<T> {
//         let mut ctx = AcquireCtx {
//             pool: self,
//             top_identity: TypeId::of::<T::Identity>(),
//         };
//         T::try_from_pool(&mut ctx)
//     }
//
//     #[inline]
//     fn release<T: IntoPool>(&mut self, obj: T) {
//         let mut ctx = ReleaseCtx {
//             pool: self,
//             top_identity: TypeId::of::<T::Identity>(),
//         };
//         obj.into_pool(&mut ctx);
//
//         // After releasing a top-level object, we prune stale allocations.
//         self.arrays.prune_stale();
//     }
//
//     #[inline]
//     fn clear(&mut self) {
//         self.objects.clear();
//         self.arrays.clear();
//     }
//
//     #[inline]
//     fn optimize(&mut self) {
//         self.objects.optimize();
//         self.arrays.prune_stale();
//     }
// }
