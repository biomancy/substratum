use crate::memory::ArrayAllocation;
use std::any::TypeId;
use std::collections::HashMap;

enum StealOutcome {
    Success(ArrayAllocation),
    Fail,
    CacheEviction,
}

// The core for a T::Identity, holding metadata and arrays created for objects with a particular
// TypeId, all having identical element size and alignment.
#[derive(Debug)]
struct AffinityCache {
    allocations: Vec<ArrayAllocation>,
    // Max number of allocations before spilling over to the global pool.
    max_size: usize,
    // Max number of failed steal attempts before this core is marked for eviction.
    eviction_thr: usize,
    // Counter for failed steal attempts. Reset by any 'release' to this core.
    failed_steals: usize,
}

// SAFETY: AffinityCache stores raw allocation pointers, but it does not
// expose any methods that would allow concurrent access by enforcing
// exclusive (&mut self) access to its methods.
unsafe impl Send for AffinityCache {}

impl Default for AffinityCache {
    fn default() -> Self {
        AffinityCache {
            allocations: Vec::new(),
            max_size: 16,
            eviction_thr: 8,
            failed_steals: 0,
        }
    }
}

impl AffinityCache {
    #[inline]
    fn put(&mut self, allocation: ArrayAllocation) -> Option<ArrayAllocation> {
        self.failed_steals = 0; // Reset the TTL counter on any successful put.

        if self.allocations.len() < self.max_size {
            self.allocations.push(allocation);
            None
        } else {
            Some(allocation)
        }
    }

    #[inline]
    fn take(&mut self) -> Option<ArrayAllocation> {
        self.allocations.pop()
    }

    #[inline]
    fn steal(&mut self) -> StealOutcome {
        if let Some(object) = self.allocations.pop() {
            return StealOutcome::Success(object);
        }
        self.failed_steals = (self.failed_steals + 1) % (self.eviction_thr + 1);
        if self.failed_steals != self.eviction_thr {
            StealOutcome::Fail
        } else {
            // If we reach the max_ttl for the first time, we should mark this core for eviction.
            // This will happen only once, and then the counter will stay at max_ttl + 1.
            StealOutcome::CacheEviction
        }
    }

    #[inline]
    fn is_stale(&self) -> bool {
        debug_assert!({
            self.failed_steals < self.eviction_thr
                || (self.allocations.is_empty()
                    && (self.failed_steals == self.eviction_thr
                        || self.failed_steals == self.eviction_thr + 1))
        });
        self.allocations.is_empty()
            && (self.failed_steals == self.eviction_thr + 1
                || self.failed_steals == self.eviction_thr)
    }

    #[inline]
    fn evict_after(&mut self, eviction_thr: usize) {
        self.eviction_thr = eviction_thr;
        // Ensure the TTL counter does not exceed the new max_ttl.
        // This will prevent core from becoming stale immediately after setting a new max_ttl.
        self.failed_steals = std::cmp::max(self.failed_steals, eviction_thr).saturating_sub(1);
        debug_assert!(self.failed_steals < self.eviction_thr || self.eviction_thr == 0);
    }

    #[inline]
    fn spill_after(&mut self, max_size: usize) {
        self.max_size = max_size;
    }
}

// The global pool for all arrays with a specific (alignment, element_size).

#[derive(Debug)]
struct StealingCache {
    // Round-robin list of TypeIds for all AffinityCaches that can use this global pool.
    stealing_ring: Vec<TypeId>,
    next_stealing_idx: usize,
    // The shared pool of spilled-over allocations.
    max_size: usize,
    allocations: Vec<ArrayAllocation>,
}

// SAFETY: StealingCache stores raw allocation pointers, but it does not
// expose any methods that would allow concurrent access by enforcing
// exclusive (&mut self) access to its methods.
unsafe impl Send for StealingCache {}

impl StealingCache {
    #[inline]
    fn put(&mut self, allocation: ArrayAllocation) -> Option<ArrayAllocation> {
        // Discard the allocation if the core is over or at max_size.
        if self.allocations.len() < self.max_size {
            self.allocations.push(allocation);
            None
        } else {
            Some(allocation)
        }
    }

    #[inline]
    fn take(&mut self) -> Option<ArrayAllocation> {
        self.allocations.pop()
    }

    #[inline]
    fn nest_stealing_target(&mut self) -> &TypeId {
        debug_assert!(!self.stealing_ring.is_empty());

        // SAFETY: The stealing_ring is guaranteed to be non-empty by the core invariant.
        let steal_from = unsafe {
            self.stealing_ring
                .get(self.next_stealing_idx)
                .unwrap_unchecked()
        };
        self.next_stealing_idx = (self.next_stealing_idx + 1) % self.stealing_ring.len();
        steal_from
    }

    #[inline]
    fn discard_after(&mut self, max_size: usize) {
        self.max_size = max_size;
    }

    #[inline]
    fn add_stealing_target(&mut self, affinity_id: TypeId) {
        // The affinity ID is guaranteed to be unique in the stealing ring.
        debug_assert!(!self.stealing_ring.contains(&affinity_id));
        self.stealing_ring.push(affinity_id);
    }

    #[inline]
    fn remove_stealing_target(&mut self, affinity_id: &TypeId) -> bool {
        // The affinity ID is guaranteed to be in the stealing ring by the invariant of the pool.
        debug_assert!(self.stealing_ring.contains(affinity_id));
        let pos = unsafe {
            self.stealing_ring
                .iter()
                .position(|id| id == affinity_id)
                .unwrap_unchecked()
        };
        self.stealing_ring.swap_remove(pos);
        self.next_stealing_idx = self.next_stealing_idx.saturating_sub(1);

        self.stealing_ring.is_empty()
    }
}

// The top-level manager for all array pooling.
#[derive(Default, Debug)]
pub struct AffinityArrayCache {
    // The primary, affinity-based core.
    // The key is (T::Identity::TypeId, Alignment, Element size).
    affine: HashMap<(TypeId, usize, usize), AffinityCache>,

    // The secondary, global core for physically compatible arrays.
    // The key is (Alignment, Element size).
    global: HashMap<(usize, usize), StealingCache>,

    // Queue of core (identified by their key) to check for eviction after finishing each
    // 'release' calls.
    eviction_queue: Vec<(TypeId, usize, usize)>,
}

impl AffinityArrayCache {
    pub fn take(
        &mut self,
        affinity_id: &TypeId,
        alignment: usize,
        element_size: usize,
    ) -> Option<ArrayAllocation> {
        // Take is a fast path that tries to:
        // 1. Find an allocation in the affinity core for the given TypeId.
        // 2. If not found, try to take spilled allocations from the global pool.
        // 3. If not available, attempt to steal from the next core in the round-robin list.
        // 4. If the stealing attempt fails, bump the TTL counter and then return None.

        if let Some(array) = self
            .affine
            .get_mut(&(*affinity_id, alignment, element_size))
            .and_then(|a| a.take())
        {
            return Some(array);
        }

        let global_cache = self.global.get_mut(&(alignment, element_size))?;
        if let Some(array) = global_cache.take() {
            return Some(array);
        }

        // Global core miss, try to steal from the next core in the round-robin list.
        let stealing_id = global_cache.nest_stealing_target();

        // SAFETY: The stealing_target is guaranteed to be valid as it comes from the
        // round-robin list of TypeIds. This is the pool invariant upheld by the release logic.
        let stealing_target = unsafe {
            self.affine
                .get_mut(&(*stealing_id, alignment, element_size))
                .unwrap_unchecked()
        };

        match stealing_target.steal() {
            StealOutcome::Success(x) => Some(x),
            StealOutcome::Fail => None,
            StealOutcome::CacheEviction => {
                // If the core is marked for eviction we must add it to the eviction queue.
                // Each core can be marked as evicted only once.
                debug_assert!(stealing_target.is_stale());
                debug_assert!(!self.eviction_queue.contains(&(
                    *stealing_id,
                    alignment,
                    element_size
                )));
                self.eviction_queue
                    .push((*stealing_id, alignment, element_size));
                None
            }
        }
    }

    pub fn put(
        &mut self,
        affinity_id: &TypeId,
        alignment: usize,
        element_size: usize,
        allocation: ArrayAllocation,
    ) {
        // Put is a fast path that tries to:
        // 1. Put the allocation into the affinity core for the given TypeId.
        // 2. If the core is full, spill over to the global pool.
        // 3. If the global pool is full, discard the allocation.
        todo!("Correctly add the type to the round-robin list of the global cache, when needed.");

        let allocation = self
            .affine
            .entry((*affinity_id, alignment, element_size))
            .or_default()
            .put(allocation);
        let allocation = match allocation {
            None => return,
            Some(allocation) => allocation,
        };

        let global_key = (alignment, element_size);
        let allocation = self
            .global
            .entry(global_key)
            .or_insert_with(|| {
                StealingCache {
                    stealing_ring: vec![*affinity_id],
                    next_stealing_idx: 0,
                    max_size: 256, // Default max size for the global core.
                    allocations: Vec::new(),
                }
            })
            .put(allocation);

        match allocation {
            None => return,
            Some(x) => std::mem::drop(x),
        }
    }

    pub fn prune_stale(&mut self) {
        // Remove stale core from the global pool.
        for (affinity_id, alignment, element_size) in self.eviction_queue.drain(..) {
            debug_assert!(
                self.affine
                    .contains_key(&(affinity_id, alignment, element_size))
            );
            // SAFETY: The key is guaranteed to be valid as it comes from the eviction queue.
            let is_still_stale = unsafe {
                self.affine
                    .get(&(affinity_id, alignment, element_size))
                    .unwrap_unchecked()
                    .is_stale()
            };

            // If the core was repopulated and is no longer stale, we skip it.
            if !is_still_stale {
                continue;
            }
            self.affine.remove(&(affinity_id, alignment, element_size));

            // Drop the corresponding global core record.
            // SAFETY: The key is guaranteed to be valid as it comes from the eviction queue.
            let is_global_empty = unsafe {
                self.global
                    .get_mut(&(alignment, element_size))
                    .unwrap_unchecked()
                    .remove_stealing_target(&affinity_id)
            };

            // If the global core is empty, we can remove it entirely.
            if is_global_empty {
                self.global.remove(&(alignment, element_size));
            }
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        self.affine.clear();
        self.global.clear();
        self.eviction_queue.clear();
    }

    #[inline]
    pub fn configure_affine_cache(
        &mut self,
        affinity_id: &TypeId,
        alignment: usize,
        element_size: usize,
        max_size: usize,
        eviction_threshold: usize,
    ) {
        self.affine
            .entry((*affinity_id, alignment, element_size))
            .and_modify(|x| {
                x.spill_after(max_size);
                x.evict_after(eviction_threshold);
            });
    }

    #[inline]
    pub fn configure_global_cache(
        &mut self,
        alignment: usize,
        element_size: usize,
        max_size: usize,
    ) {
        self.global
            .entry((alignment, element_size))
            .and_modify(|x| x.discard_after(max_size));
    }
}
