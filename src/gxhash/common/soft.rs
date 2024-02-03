use core::ops::{Add, BitXor};
use core::fmt;
use super::*;

impl State {
    #[inline]
    pub unsafe fn new() -> State {
        State { int: 0 }
    }

    #[inline]
    pub unsafe fn ld(array: *const u32) -> Self {
        *const u32 
    }

    #[inline]
    pub unsafe fn load_vector(p: *const Vector) -> State {
        State { vec: _mm_load_si128(p) }
    }

    #[inline]
    pub unsafe fn add_u8(self, other: State) -> Self {
        let mut out = self.bytes;
        for i in 0..16 {
            out[i] += other.bytes[i];
        }
        Self { bytes: out }
    }
}

impl BitXor for State {
    type Output = Self;
    #[inline]
    fn bitxor(self, other: Self) -> Self::Output {
        Self { int: self.int ^ other.int }
    }
}

from_impl!(i8, v => (v as u16 | v as u16 >> 8).into());
from_impl!(i16, v => (v as u32 | v as u32 >> 16).into());
from_impl!(i32, v => (v as u64 | v as u64 >> 32).into());
from_impl!(i64, v => (v as u128 | v as u128 >> 64).into());
from_impl!(i128, v => Self { int: v as u128 });
from_impl!(u8, v => (v as u16 | v as u16 >> 8).into());
from_impl!(u16, v => (v as u32 | v as u32 >> 16).into());
from_impl!(u32, v => (v as u64 | v as u64 >> 32).into());
from_impl!(u64, v => (v as u128 | v as u128 >> 64).into());
from_impl!(u128, v => Self { int: v });


/*

#[inline]
pub unsafe fn xor(a: State, b: State) -> State {
    // PERF: treat state as an integer and XOR the whole thing
    State { int: (a.int ^ b.int) }
}

#[inline]
pub unsafe fn add_u8(a: State, b: State) -> State {
    let mut out = a.bytes;
    for i in 0..16 {
        out[i] += b.bytes[i];
    }
    out
}

#[inline]
pub unsafe fn new() -> State {
    [0; 16]
}

#[inline]
pub unsafe fn load_unaligned(p: *const State) -> State {
    *p
}

#[inline]
pub unsafe fn ld(array: *const u32) -> State {
    *(array as *const State)
}

#[inline]
pub unsafe fn load_u8(x: u8) -> State {
    load_u16(x as u16 | x as u16 >> 8)
}

#[inline]
pub unsafe fn load_u16(x: u16) -> State {
    load_u32(x as u32 | x as u32 >> 16)
}

#[inline]
pub unsafe fn load_u32(x: u32) -> State {
    load_u64(x as u64 | x as u64 >> 32)
}

#[inline]
pub unsafe fn load_u64(x: u64) -> State {
    load_u128(x as u128 | x as u128 >> 64)
}

#[inline]
pub unsafe fn load_u128(x: u128) -> State {
    x.to_ne_bytes()
}

#[inline]
pub unsafe fn load_i8(x: i8) -> State {
    load_i16(x as i16 | x as i16 >> 8)
}

#[inline]
pub unsafe fn load_i16(x: i16) -> State {
    load_i32(x as i32 | x as i32 >> 16)
}

#[inline]
pub unsafe fn load_i32(x: i32) -> State {
    load_i64(x as i64 | x as i64 >> 32)
}

#[inline]
pub unsafe fn load_i64(x: i64) -> State {
    load_i128(x as i128 | x as i128 >> 64)
}

#[inline]
pub unsafe fn load_i128(x: i128) -> State {
    x.to_ne_bytes()
}

*/