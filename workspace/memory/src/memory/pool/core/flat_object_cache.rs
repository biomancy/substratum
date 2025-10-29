use crate::memory::ObjectAllocation;
use std::alloc::Layout;
use std::collections::HashMap;

#[derive(Default, Debug)]
pub struct FlatObjectCache {
    inner: HashMap<Layout, Vec<ObjectAllocation>>,
}

// SAFETY: FlatObjectCache stores raw allocation pointers, but it does not
// expose any methods that would allow concurrent access by enforcing
// exclusive (&mut self) access to its methods.
unsafe impl Sync for FlatObjectCache {}

impl FlatObjectCache {
    #[inline]
    pub fn put(&mut self, allocation: ObjectAllocation) {
        let layout = allocation.layout();
        self.inner.entry(*layout).or_default().push(allocation);
    }

    #[inline]
    pub fn take(&mut self, layout: &Layout) -> Option<ObjectAllocation> {
        if let Some(allocations) = self.inner.get_mut(layout) {
            if let Some(allocation) = allocations.pop() {
                return Some(allocation);
            }
        }
        None
    }

    #[inline]
    pub fn optimize(&mut self) {
        self.inner.retain(|_, allocations| !allocations.is_empty());
    }

    #[inline]
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    #[inline]
    pub fn into_inner(self) -> HashMap<Layout, Vec<ObjectAllocation>> {
        self.inner
    }
}
