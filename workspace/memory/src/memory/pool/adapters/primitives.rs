use crate::memory::pool::lifecycle::{FromPool, IntoPool, PoolAcquireHandle, PoolReleaseHandle, TryFromPool};

macro_rules! primitive_pool_adaptor {
    () => {};
    ($type:ty, $($tail: ty,)*) => {
        primitive_pool_adaptor!($($tail,)*);

        impl IntoPool for $type {
            type Identity = $type;

            #[inline]
            fn into_pool(self, _handle: &mut impl PoolReleaseHandle) {}
        }

        impl FromPool for $type {
            #[inline]
            fn from_pool(_handle: &mut impl PoolAcquireHandle) -> Self {
                Self::default()
            }
        }

        impl TryFromPool for $type {
            #[inline]
            fn try_from_pool(_handle: &mut impl PoolAcquireHandle) -> Option<Self> {
                Some(Self::default())
            }
        }
    };
}

// According to the Rust documentation (https://doc.rust-lang.org/reference/types.html),
// the primitive types are:
// * Boolean: `bool`
// * Numeric:
//     * Signed integers:`i8`, `i16`, `i32`, `i64`, `i128`, `isize`
//     * Unsigned integers:`u8`, `u16`, `u32`, `u64`, `u128`, `usize`
//     * Float: `f32`, `f64`
// * Textual: `char`, `str`
// * Never: `!`
//
// For these primitive types, which are `Copy` + `Default`, pooling is a trivial
// operation. `IntoPool` is a no-op because these types do not own heap memory.
// `FromPool` and `TryFromPool` simply return a default value from the stack.
//
// This implementation does not cover `str` (dynamically sized) or `!` (experimental).
primitive_pool_adaptor!(
    bool, i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize, f32, f64, char,
);
