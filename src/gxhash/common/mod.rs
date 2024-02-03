pub mod soft;

#[cfg(not(any(all(nightly, target_arch = "arm"), target_arch = "aarch64", target_arch = "x86", target_arch = "x86_64")))]
pub use soft as platform;

#[cfg(any(all(nightly, target_arch = "arm"), target_arch = "aarch64"))]
#[path = "arm.rs"]
pub mod platform;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[path = "x86.rs"]
pub mod platform;

// BYTES - 16-byte aligned array of 16 bytes; used to represent state as bytes

use core::ops::{Deref, DerefMut};

#[derive(Clone, Copy, Debug)]
#[repr(align(16))]
pub struct Bytes([u8; 16]);

impl Deref for Bytes {
    type Target = [u8; 16];
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Bytes {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// STATE

// SAFETY: These are all the same shape in memory.
#[repr(C)]
#[derive(Clone, Copy)]
pub union State {
    bytes: Bytes, // slow bytewise operations
    int: u128, // faster bitwise operations
    #[cfg(any(all(nightly, target_arch = "arm"), target_arch = "aarch64", target_arch = "x86", target_arch = "x86_64"))]
    vec: Vector, // fast SIMD operations
}

impl fmt::Debug for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "State [...]")
    }
}

macro_rules! from_impl {
    ($type:ty, $value:ident => $expr:expr) => {
        impl From<$type> for State {
            #[inline]
            fn from($value: $type) -> Self {
                $expr
            }
        }
    }
}

from_impl!(i128, v => Self { int: v as u128 });
from_impl!(u128, v => Self { int: v });