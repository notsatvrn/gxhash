pub use crate::gxhash::common::platform::*;

use super::*;

#[inline]
#[target_feature(enable = "aes")]
// See https://blog.michaelbrase.com/2018/05/08/emulating-x86-aes-intrinsics-on-armv8-a
pub unsafe fn aes_encrypt(data: State, keys: State) -> State {
    // Encrypt
    let encrypted = vaeseq_u8(vreinterpretq_u8_s8(data), vdupq_n_u8(0));
    // Mix columns
    let mixed = vaesmcq_u8(encrypted);
    // Xor keys
    vreinterpretq_s8_u8(veorq_u8(mixed, vreinterpretq_u8_s8(keys)))
}
    
#[inline]
#[target_feature(enable = "aes")]
// See https://blog.michaelbrase.com/2018/05/08/emulating-x86-aes-intrinsics-on-armv8-a
pub unsafe fn aes_encrypt_last(data: State, keys: State) -> State {
    // Encrypt
    let encrypted = vaeseq_u8(vreinterpretq_u8_s8(data), vdupq_n_u8(0));
    // Xor keys
    vreinterpretq_s8_u8(veorq_u8(encrypted, vreinterpretq_u8_s8(keys)))
}

#[inline]
#[target_feature(enable = "neon")]
pub unsafe fn get_partial_safe(data: *const State, len: usize) -> State {
    // Temporary buffer filled with zeros
    let mut buffer = [0i8; 16];
    // Copy data into the buffer
    std::ptr::copy(data as *const i8, buffer.as_mut_ptr(), len);
    // Load the buffer into a vector
    let partial_vector = vld1q_s8(buffer.as_ptr());
    vaddq_s8(partial_vector, vdupq_n_s8(len as i8))
}

#[inline]
#[target_feature(enable = "neon")]
pub unsafe fn get_partial_unsafe(data: *const State, len: usize) -> State {
    let indices = vld1q_s8([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15].as_ptr());
    let mask = vcgtq_s8(vdupq_n_s8(len as i8), indices);
    let partial_vector = vandq_s8(load_unaligned(data), vreinterpretq_s8_u8(mask));
    vaddq_s8(partial_vector, vdupq_n_s8(len as i8))
}
