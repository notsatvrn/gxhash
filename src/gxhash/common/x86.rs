#[cfg(target_arch = "x86")]
pub use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
pub use core::arch::x86_64::*;

use core::ops::{Add, BitXor};
use core::fmt;
use super::*;

pub type Vector = __m128i;

impl State {
    #[inline]
    pub unsafe fn new() -> State {
        State { int: 0 }
    }

    #[inline]
    #[target_feature(enable = "sse2")]
    pub unsafe fn ld(array: *const u32) -> Self {
        State { vec: _mm_loadu_si128(array as *const Vector) }
    }

    #[inline]
    #[target_feature(enable = "sse2")]
    pub unsafe fn load_vector(p: *const Vector) -> State {
        State { vec: _mm_load_si128(p) }
    }

    #[inline]
    #[target_feature(enable = "sse2")]
    pub unsafe fn add_u8(a: State, b: State) -> Self {
        State { vec: _mm_add_epi8(a.vec, b.vec) }
    }
}

impl BitXor for State {
    type Output = Self;
    #[inline]
    #[target_feature(enable = "sse2")]
    fn bitxor(self, other: Self) -> Self::Output {
        Self { vec: _mm_xor_si128(self.vec, other.vec) }
    }
}

macro_rules! from_impl {
    ($type:ty, $value:ident => $expr:expr) => {
        impl From<$type> for State {
            #[inline]
            fn from($value: $type) -> Self {
                unsafe { $expr }
            }
        }
    }
}

from_impl!(i8, v => Self { vec: _mm_set1_epi8(v) });
from_impl!(i16, v => Self { vec: _mm_set1_epi16(v) });
from_impl!(i32, v => Self { vec: _mm_set1_epi32(v) });
from_impl!(i64, v => Self { vec: _mm_set1_epi64x(v) });
from_impl!(i128, v => Self { int: v as i128 });
from_impl!(u8, v => Self { vec: _mm_set1_epi8(v as u8) });
from_impl!(u16, v => Self { vec: _mm_set1_epi16(v as u16) });
from_impl!(u32, v => Self { vec: _mm_set1_epi32(v as u32) });
from_impl!(u64, v => Self { vec: _mm_set1_epi64x(v as u64) });
from_impl!(u128, v => Self { int: v });
