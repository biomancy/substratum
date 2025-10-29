use std::alloc::Layout;
use std::ptr::NonNull;

/// A memory allocation previously used for a single object.
///
/// This type owns a raw pointer and its corresponding `Layout`, and will deallocate
/// the memory on drop. It is distinct from [`ArrayAllocation`] to enable specialized,
/// more efficient pooling strategies for fixed-size objects.
#[derive(Debug)]
pub struct ObjectAllocation {
    ptr: NonNull<u8>,
    layout: Layout,
}

// SAFETY: `ObjectAllocation` owns the pointer. Since the data it points to is not
// accessed non-atomically and the pointer itself is `Send`, it's safe to send
// the allocation across threads.
unsafe impl Send for ObjectAllocation {}
impl Drop for ObjectAllocation {
    fn drop(&mut self) {
        // SAFETY: The pointer and layout are guaranteed to be valid and paired
        // correctly by the constructor's safety contract.
        unsafe { std::alloc::dealloc(self.ptr.as_ptr(), self.layout) }
    }
}

impl ObjectAllocation {
    /// Creates a new `ObjectAllocation` from a raw pointer and its layout.
    ///
    /// # Safety
    /// The caller must ensure the pointer was allocated using the given `layout`
    /// and that it is unique.
    #[inline]
    pub unsafe fn new(ptr: NonNull<u8>, layout: Layout) -> Self {
        ObjectAllocation { ptr, layout }
    }

    /// Returns the layout of the allocation.
    #[inline]
    pub fn layout(&self) -> &Layout {
        &self.layout
    }

    /// Consumes the `ObjectAllocation`, returning the owned pointer and layout.
    ///
    /// This bypasses the `Drop` implementation, making the caller responsible
    /// for deallocating the memory.
    #[inline]
    #[must_use]
    pub fn into_inner(self) -> (NonNull<u8>, Layout) {
        let me = std::mem::ManuallyDrop::new(self);
        (me.ptr, me.layout)
    }
}

/// A memory allocation previously used for an array.
///
/// This type owns a raw pointer and its corresponding array layout, and will
/// deallocate the memory on drop. The allocation capacity is guaranteed to be non-zero.
#[derive(Debug)]
pub struct ArrayAllocation {
    ptr: NonNull<u8>,
    element_size: usize,
    layout: Layout,
}

// SAFETY: `ArrayAllocation` owns the pointer. Since the data it points to is not
// accessed non-atomically and the pointer itself is `Send`, it's safe to send
// the allocation across threads.
unsafe impl Send for ArrayAllocation {}

impl Drop for ArrayAllocation {
    fn drop(&mut self) {
        // SAFETY: The pointer and layout are guaranteed to be valid and paired
        // correctly by the constructor's safety contract.
        unsafe { std::alloc::dealloc(self.ptr.as_ptr(), self.layout) }
    }
}

impl ArrayAllocation {
    /// Creates a new `ArrayAllocation` from its raw parts.
    ///
    /// # Safety
    /// The caller must ensure that:
    /// * The pointer is unique and valid for the given `layout`.
    /// * The `layout` is an array layout with a non-zero capacity.
    /// * The `element_size` must be > 0 and divide the `layout.size()` evenly.
    pub unsafe fn new(ptr: NonNull<u8>, element_size: usize, layout: Layout) -> Self {
        debug_assert!(element_size > 0, "element_size must be greater than 0");
        debug_assert!(
            layout.size() % element_size == 0,
            "layout size must be divisible by element_size"
        );
        ArrayAllocation {
            ptr,
            element_size,
            layout,
        }
    }

    /// Creates a new `ArrayAllocation` by taking ownership of a `Vec`'s buffer.
    ///
    /// Returns `None` if the vector's capacity is `0`.
    pub fn from_vec<T>(mut vec: Vec<T>) -> Option<Self> {
        if vec.capacity() == 0 || size_of::<T>() == 0 {
            return None;
        }

        // Ensure the Vec is empty so no destructors are missed.
        vec.clear();

        let mut vec = std::mem::ManuallyDrop::new(vec);
        let ptr = vec.as_mut_ptr();
        let layout = Layout::array::<T>(vec.capacity()).unwrap();
        debug_assert_eq!(layout.align(), std::mem::align_of::<T>());

        Some(ArrayAllocation {
            // SAFETY: `Vec<T>` guarantees its pointer is non-null and valid
            // for the given capacity and layout. `ManuallyDrop` prevents a double-free.
            ptr: unsafe { NonNull::new_unchecked(ptr as *mut u8) },
            element_size: size_of::<T>(),
            layout,
        })
    }

    /// Returns the layout of the allocation.
    #[inline]
    pub fn layout(&self) -> &Layout {
        &self.layout
    }

    /// Returns the size of each element in the original array.
    ///
    /// The size is guaranteed to be greater than `0`.
    #[inline]
    pub fn element_size(&self) -> usize {
        self.element_size
    }

    /// Consumes the `ArrayAllocation`, returning its raw parts.
    ///
    /// This bypasses the `Drop` implementation, making the caller responsible
    /// for deallocating the memory.
    #[inline]
    #[must_use]
    pub fn into_inner(self) -> (NonNull<u8>, usize, Layout) {
        let me = std::mem::ManuallyDrop::new(self);
        (me.ptr, me.element_size, me.layout)
    }

    /// Convert the allocation into a `Vec<T>`.
    ///
    /// The conversion will fail if the `T` type is not compatible with the allocation's layout.
    pub fn into_vec<T>(self) -> Result<Vec<T>, &'static str> {
        let (ptr, _, layout) = self.into_inner();

        // Note - the alignment of `T` must match the allocation's alignment according to the Vec
        // safety contract, and the size of the allocation must be a multiple of `T`'s size.
        if align_of::<T>() != layout.align() {
            return Err("Element type alignment does not match allocation alignment");
        } else if layout.size() % size_of::<T>() != 0 {
            return Err("Allocation size is not a multiple of element size");
        }
        let capacity = layout.size() / size_of::<T>();

        // SAFETY: The pointer is guaranteed to be valid for the given capacity and element size.
        let vec = unsafe { Vec::from_raw_parts(ptr.as_ptr() as *mut T, 0, capacity) };
        Ok(vec)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    mod object_allocation {
        use super::*;

        #[test]
        fn drop_deallocates_memory() {
            let layout = Layout::new::<u32>();
            // SAFETY: The pointer is allocated with the correct layout.
            let ptr = unsafe { std::alloc::alloc(layout) };
            // SAFETY: The pointer and layout are valid and unique.
            let allocation = unsafe { ObjectAllocation::new(NonNull::new(ptr).unwrap(), layout) };

            assert_eq!(allocation.layout(), &layout);
            // `allocation` is dropped here, and its memory should be deallocated.
        }

        #[test]
        fn into_inner_prevents_drop() {
            let layout = Layout::new::<u64>();
            let ptr = unsafe { std::alloc::alloc(layout) };
            let allocation = unsafe { ObjectAllocation::new(NonNull::new(ptr).unwrap(), layout) };

            // Consume the allocation to get the raw parts.
            let (inner_ptr, inner_layout) = allocation.into_inner() ;
            assert_eq!(inner_ptr.as_ptr(), ptr);
            assert_eq!(inner_layout, layout);

            // Manually deallocate the memory. If `drop` were also called, this would
            // be a double-free, which memory sanitizers would catch.
            unsafe { std::alloc::dealloc(ptr, layout) };
        }
    }

    mod array_allocation {
        use super::*;

        #[test]
        fn new_and_drop_deallocates_memory() {
            let layout = Layout::array::<u32>(10).unwrap();
            // SAFETY: The pointer is allocated with the correct layout.
            let ptr = unsafe { std::alloc::alloc(layout) };
            // SAFETY: The pointer, element size, and layout are valid and consistent.
            let allocation = unsafe {
                ArrayAllocation::new(NonNull::new(ptr).unwrap(), size_of::<u32>(), layout)
            };

            assert_eq!(allocation.element_size(), size_of::<u32>());
            assert_eq!(allocation.layout(), &layout);
            // `allocation` is dropped here, and its memory should be deallocated.
        }

        #[test]
        fn into_inner_prevents_drop() {
            let layout = Layout::array::<u16>(20).unwrap();
            // SAFETY: The pointer is allocated with the correct layout.
            let ptr = unsafe { std::alloc::alloc(layout) };
            // SAFETY: The pointer, element size, and layout are valid and consistent.
            let allocation = unsafe {
                ArrayAllocation::new(NonNull::new(ptr).unwrap(), size_of::<u16>(), layout)
            };

            // Consume the allocation to get the raw parts.
            // SAFETY: We are taking ownership of the pointer and will deallocate it manually.
            let (inner_ptr, inner_element_size, inner_layout) = allocation.into_inner();
            assert_eq!(inner_ptr.as_ptr(), ptr);
            assert_eq!(inner_element_size, size_of::<u16>());
            assert_eq!(inner_layout, layout);

            // Manually deallocate the memory.
            // SAFETY: The pointer and layout are the same ones used for allocation.
            unsafe { std::alloc::dealloc(ptr, layout) };
        }

        #[test]
        fn from_vec_with_capacity() {
            let vec = Vec::<u32>::with_capacity(10);
            let expected_layout = Layout::array::<u32>(10).unwrap();
            let allocation = ArrayAllocation::from_vec(vec).unwrap();

            assert_eq!(allocation.element_size(), size_of::<u32>());
            assert_eq!(allocation.layout(), &expected_layout);
            // `allocation` is dropped here, deallocating the vec's buffer.
        }

        #[test]
        fn from_vec_with_zero_capacity_is_none() {
            let vec = Vec::<u32>::with_capacity(0);
            assert!(ArrayAllocation::from_vec(vec).is_none());
        }

        #[test]
        fn from_vec_with_zero_sized_type_is_none() {
            let vec = Vec::<()>::with_capacity(10);
            assert!(ArrayAllocation::from_vec(vec).is_none());
        }

        #[test]
        fn into_vec_success() {
            let vec = Vec::<u32>::with_capacity(10);
            let allocation = ArrayAllocation::from_vec(vec).unwrap();

            let new_vec = allocation.into_vec::<(i32, i32)>().unwrap();
            assert_eq!(new_vec.len(), 0);
            assert_eq!(new_vec.capacity(), 5);
        }

        #[test]
        fn into_vec_mismatched_alignment() {
            // Create an allocation for u64 (align 8)
            let vec = Vec::<u64>::with_capacity(5);
            let allocation = ArrayAllocation::from_vec(vec).unwrap();

            // Try to convert to u32 (align 4), which should fail.
            let result = allocation.into_vec::<u32>();
            assert_eq!(
                result.unwrap_err(),
                "Element type alignment does not match allocation alignment"
            );
        }

        #[test]
        fn into_vec_incompatible_size() {
            let vec = Vec::<u8>::with_capacity(4);
            let allocation = ArrayAllocation::from_vec(vec).unwrap();

            let result = allocation.into_vec::<(u8, u8, u8)>();
            assert_eq!(
                result.unwrap_err(),
                "Allocation size is not a multiple of element size"
            );
        }
    }
}
