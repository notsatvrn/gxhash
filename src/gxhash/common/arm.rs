#[cfg(target_arch = "arm")]
pub use core::arch::arm::*;
#[cfg(target_arch = "aarch64")]
pub use core::arch::aarch64::*;

use super::*;

pub type Vector = int8x16_t;

#[inline]
#[target_feature(enable = "neon")]
pub unsafe fn xor(a: State, b: State) -> State {
    vreinterpretq_s8_u8(veorq_u8(a, vreinterpretq_u8_s8(b)))
}

#[inline]
#[target_feature(enable = "neon")]
pub unsafe fn add_u8(a: State, b: State) -> State {
    vaddq_s8(a, b)
}

#[inline]
#[target_feature(enable = "neon")]
pub unsafe fn new() -> State {
    vdupq_n_s8(0)
}

#[inline]
#[target_feature(enable = "neon")]
pub unsafe fn load_unaligned(p: *const State) -> State {
    vld1q_s8(p as *const i8)
}

#[inline]
#[target_feature(enable = "neon")]
pub unsafe fn ld(array: *const u32) -> State {
    vreinterpretq_s8_u32(vld1q_u32(array))
}

#[inline]
#[target_feature(enable = "neon")]
pub unsafe fn load_u8(x: u8) -> State {
    vreinterpretq_s8_u8(vdupq_n_u8(x))
}

#[inline]
#[target_feature(enable = "neon")]
pub unsafe fn load_u16(x: u16) -> State {
    vreinterpretq_s8_u16(vdupq_n_u16(x))
}

#[inline]
#[target_feature(enable = "neon")]
pub unsafe fn load_u32(x: u32) -> State {
    vreinterpretq_s8_u32(vdupq_n_u32(x))
}

#[inline]
#[target_feature(enable = "neon")]
pub unsafe fn load_u64(x: u64) -> State {
    vreinterpretq_s8_u64(vdupq_n_u64(x))
}

#[inline]
#[target_feature(enable = "neon")]
pub unsafe fn load_u128(x: u128) -> State {
    let ptr = &x as *const u128 as *const i8;
    vld1q_s8(ptr)
}

#[inline]
#[target_feature(enable = "neon")]
pub unsafe fn load_i8(x: i8) -> State {
    vdupq_n_s8(x)
}

#[inline]
#[target_feature(enable = "neon")]
pub unsafe fn load_i16(x: i16) -> State {
    vreinterpretq_s8_s16(vdupq_n_s16(x))
}

#[inline]
#[target_feature(enable = "neon")]
pub unsafe fn load_i32(x: i32) -> State {
    vreinterpretq_s8_s32(vdupq_n_s32(x))
}

#[inline]
#[target_feature(enable = "neon")]
pub unsafe fn load_i64(x: i64) -> State {
    vreinterpretq_s8_s64(vdupq_n_s64(x))
}

#[inline]
#[target_feature(enable = "neon")]
pub unsafe fn load_i128(x: i128) -> State {
    let ptr = &x as *const i128 as *const i8;
    vld1q_s8(ptr)
}