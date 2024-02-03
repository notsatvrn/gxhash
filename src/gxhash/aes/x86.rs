pub use crate::gxhash::common::platform::*;

use super::*;

#[inline]
#[target_feature(enable = "aes")]
pub unsafe fn aes_encrypt(data: State, keys: State) -> State {
    _mm_aesenc_si128(data, keys)
}

#[inline]
#[target_feature(enable = "aes")]
pub unsafe fn aes_encrypt_last(data: State, keys: State) -> State {
    _mm_aesenclast_si128(data, keys)
}

#[inline]
#[target_feature(enable = "sse2")]
pub unsafe fn get_partial_safe(data: *const State, len: usize) -> State {
    // Temporary buffer filled with zeros
    let mut buffer = Bytes::new();
    // Copy data into the buffer
    std::ptr::copy(data as *const i8, buffer.0.as_mut_ptr() as *mut i8, len);
    // Load the buffer into a vector
    let partial_vector = _mm_load_si128(buffer.0.as_ptr() as *const State);
    _mm_add_epi8(partial_vector, _mm_set1_epi8(len as i8))
}

#[inline]
#[target_feature(enable = "sse2")]
pub unsafe fn get_partial_unsafe(data: *const State, len: usize) -> State {
    let indices = _mm_set_epi8(15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0);
    let mask = _mm_cmpgt_epi8(_mm_set1_epi8(len as i8), indices);
    let partial_vector = _mm_and_si128(_mm_loadu_si128(data), mask);
    _mm_add_epi8(partial_vector, _mm_set1_epi8(len as i8))
}

#[inline]
#[cfg(nightly)]
#[target_feature(enable = "vaes")]
pub unsafe fn compress_8_hybrid(ptr: *const State, end_address: usize, hash_vector: State, len: usize) -> State {
    macro_rules! load_unaligned_x2 {
        ($ptr:ident, $($var:ident),+) => {
            $(
                #[allow(unused_mut)]
                let mut $var = _mm256_loadu_si256($ptr);
                $ptr = ($ptr).offset(1);
            )+
        };
    }
    
    let mut ptr = ptr as *const __m256i;
    let mut t = _mm256_setzero_si256();
    let mut lane = _mm256_set_m128i(hash_vector, hash_vector);
    while (ptr as usize) < end_address {

        load_unaligned_x2!(ptr, v0, v1, v2, v3);

        let mut tmp = _mm256_aesenc_epi128(v0, v1);
        tmp = _mm256_aesenc_epi128(tmp, v2);
        tmp = _mm256_aesenc_epi128(tmp, v3);

        t = _mm256_add_epi8(t, _mm256_loadu_si256(KEYS.as_ptr() as *const __m256i));

        lane = _mm256_aesenclast_epi128(_mm256_aesenc_epi128(tmp, t), lane);
    }
    // Extract the two 128-bit lanes
    let mut lane1 = _mm256_castsi256_si128(lane);
    let mut lane2 = _mm256_extracti128_si256(lane, 1);
    // For 'Zeroes' test
    let len_vec =  _mm_set1_epi32(len as i32);
    lane1 = _mm_add_epi8(lane1, len_vec);
    lane2 = _mm_add_epi8(lane2, len_vec);
    // Merge lanes
    aes_encrypt(lane1, lane2)
}
