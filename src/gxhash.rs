// For ARM architecture
#[cfg(target_arch = "aarch64")]
mod platform_defs {
    use std::mem::size_of;
    use core::arch::aarch64::*;

    pub type state = int8x16_t;

    #[repr(C)]
    union ReinterpretUnion {
        int64: int64x2_t,
        int32: int32x4_t,
        uint32: uint32x4_t,
        int8: int8x16_t,
        uint8: uint8x16_t,
    }

    #[inline]
    pub unsafe fn create_empty() -> state {
        vdupq_n_s8(0)
    }
    
    #[inline]
    pub unsafe fn prefetch(p: *const state) {
        //__pld(p as *const i8);
    }

    #[inline]
    pub unsafe fn load_unaligned(p: *const state) -> state {
        vld1q_s8(p as *const i8)
    }

    #[inline]
    pub unsafe fn get_partial(p: *const state, len: isize) -> state {
        const MASK: [u8; size_of::<state>() * 2] = [
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00 ];

        let mask = vld1q_s8((MASK.as_ptr() as *const i8).offset(size_of::<state>() as isize - len));
        vandq_s8(load_unaligned(p), mask)
    }

    #[inline]
    pub unsafe fn compress(a: state, b: state) -> state {
        ReinterpretUnion{ uint8: aes_encrypt_last(
            ReinterpretUnion{ int8: a }.uint8, 
            ReinterpretUnion{ int8: b }.uint8) }.int8
    }

    #[inline]
    // See https://blog.michaelbrase.com/2018/05/08/emulating-x86-aes-intrinsics-on-armv8-a
    unsafe fn aes_encrypt(data: uint8x16_t, keys: uint8x16_t) -> uint8x16_t {
        // Encrypt
        let encrypted = vaeseq_u8(data, vdupq_n_u8(0));
        // Mix columns
        let mixed = vaesmcq_u8(encrypted);
        // Xor keys
        veorq_u8(mixed, keys)
    }

    #[inline]
    // See https://blog.michaelbrase.com/2018/05/08/emulating-x86-aes-intrinsics-on-armv8-a
    unsafe fn aes_encrypt_last(data: uint8x16_t, keys: uint8x16_t) -> uint8x16_t {
        // Encrypt
        let encrypted = vaeseq_u8(data, vdupq_n_u8(0));
        // Xor keys
        veorq_u8(encrypted, keys)
    }

    #[inline]
    pub unsafe fn finalize(hash: state) -> u32 {
        // Hardcoded AES keys
        let salt1 = vld1q_u32([0x713B01D0, 0x8F2F35DB, 0xAF163956, 0x85459F85].as_ptr());
        let salt2 = vld1q_u32([0x1DE09647, 0x92CFA39C, 0x3DD99ACA, 0xB89C054F].as_ptr());
        let salt3 = vld1q_u32([0xC78B122B, 0x5544B1B7, 0x689D2B7D, 0xD0012E32].as_ptr());

        // 3 rounds of AES
        let mut hash = ReinterpretUnion{ int8: hash }.uint8;
        hash = aes_encrypt(hash, ReinterpretUnion{ uint32: salt1 }.uint8);
        hash = aes_encrypt(hash, ReinterpretUnion{ uint32: salt2 }.uint8);
        hash = aes_encrypt_last(hash, ReinterpretUnion{ uint32: salt3 }.uint8);
        let hash = ReinterpretUnion{ uint8: hash }.int8;

        // Truncate to output hash size
        let p = &hash as *const state as *const u32;
        *p
    }
}

// For x86 architecture
#[cfg(target_arch = "x86_64")]
mod platform_defs {
    use core::arch::x86_64::*;
    use std::mem::size_of;

    pub type state = __m256i;

    #[inline]
    pub unsafe fn create_empty() -> state {
        _mm256_setzero_si256()
    }

    #[inline]
    pub unsafe fn prefetch(p: *const state) {
        _mm_prefetch(p as *const i8, 3);
    }

    #[inline]
    pub unsafe fn load_unaligned(p: *const state) -> state {
        _mm256_loadu_si256(p)
    }

    #[inline]
    pub unsafe fn get_partial(p: *const state, len: isize) -> state {
        const MASK: [u8; size_of::<state>() * 2] = [
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00 ];

        // Safety check
        if check_same_page(p) { // false {//
            let mask = _mm256_loadu_epi8((MASK.as_ptr() as *const i8).offset(32 - len));
            _mm256_and_si256(_mm256_loadu_si256(p), mask)
        } else {
            get_partial_safe(p as *const u8, len as usize)
        }
    }

    #[inline]
    unsafe fn check_same_page(ptr: *const state) -> bool {
        let address = ptr as usize;
        // Mask to keep only the last 12 bits (3 bytes)
        let offset_within_page = address & 0xFFF;
        // Check if the 32nd byte from the current offset exceeds the page boundary
        offset_within_page <= (4096 - size_of::<state>() - 1)
    }

    #[inline]
    unsafe fn get_partial_safe(data: *const u8, len: usize) -> state {
        // Temporary buffer filled with zeros
        let mut buffer = [0u8; size_of::<state>()];
        // Copy data into the buffer
        std::ptr::copy(data, buffer.as_mut_ptr(), len);
        // Load the buffer into a __m256i vector
        _mm256_loadu_si256(buffer.as_ptr() as *const state)
    }

    #[inline]
    pub unsafe fn compress(a: state, b: state) -> state {
        //let sum: state = _mm256_add_epi8(a, b);
        //_mm256_alignr_epi8(sum, sum, 1)
        _mm256_aesdeclast_epi128(a, b)
    }

    #[inline]
    #[allow(overflowing_literals)]
    pub unsafe fn finalize(hash: state) -> u32 {
        // Xor 256 state into 128 bit state for AES
        let lower = _mm256_castsi256_si128(hash);
        let upper = _mm256_extracti128_si256(hash, 1);
        let mut hash = _mm_xor_si128(lower, upper);

        // Hardcoded AES keys
        let salt1 = _mm_set_epi32(0x713B01D0, 0x8F2F35DB, 0xAF163956, 0x85459F85);
        let salt2 = _mm_set_epi32(0x1DE09647, 0x92CFA39C, 0x3DD99ACA, 0xB89C054F);
        let salt3 = _mm_set_epi32(0xC78B122B, 0x5544B1B7, 0x689D2B7D, 0xD0012E32);

        // 3 rounds of AES
        hash = _mm_aesenc_si128(hash, salt1);
        hash = _mm_aesenc_si128(hash, salt2);
        hash = _mm_aesenclast_si128(hash, salt3);

        // Truncate to output hash size
        let p = &hash as *const __m128i as *const u32;
        *p
    }
}

use std::intrinsics::likely;

pub use platform_defs::*;

#[inline] // To be disabled when profiling
pub fn gxhash(input: &[u8]) -> u32 {
    unsafe {
        const VECTOR_SIZE: isize = std::mem::size_of::<state>() as isize;
        
        let len: isize = input.len() as isize;
    
        let p = input.as_ptr() as *const i8;
        let mut v = p as *const state;
        let mut end_address: usize;
        let mut remaining_blocks_count: isize = len / VECTOR_SIZE;
        let mut hash_vector: state = create_empty();

        macro_rules! load_unaligned {
            ($($var:ident),+) => {
                $(
                    #[allow(unused_mut)]
                    let mut $var = load_unaligned(v);
                    v = v.offset(1);
                )+
            };
        }

        const UNROLL_FACTOR: isize = 8;
        if len >= VECTOR_SIZE * UNROLL_FACTOR {

            let unrollable_blocks_count: isize = (len / (VECTOR_SIZE * UNROLL_FACTOR)) * UNROLL_FACTOR; 
            end_address = v.offset(unrollable_blocks_count) as usize;
    
            load_unaligned!(s0, s1, s2, s3, s4, s5, s6, s7);
 
            while (v as usize) < end_address {
                
                load_unaligned!(v0, v1, v2, v3, v4, v5, v6, v7);

                prefetch(v);

                s0 = compress(s0, v0);
                s1 = compress(s1, v1);
                s2 = compress(s2, v2);
                s3 = compress(s3, v3);
                s4 = compress(s4, v4);
                s5 = compress(s5, v5);
                s6 = compress(s6, v6);
                s7 = compress(s7, v7);
            }
        
            let a = compress(compress(s0, s1), compress(s2, s3));
            let b = compress(compress(s4, s5), compress(s6, s7));
            hash_vector = compress(a, b);

            remaining_blocks_count -= unrollable_blocks_count;
        }

        end_address = v.offset(remaining_blocks_count) as usize;

        while likely((v as usize) < end_address) {
            load_unaligned!(v0);
            hash_vector = compress(hash_vector, v0);
        }

        let remaining_bytes = len & (VECTOR_SIZE - 1);
        if likely(remaining_bytes > 0) {
            let partial_vector = get_partial(v, remaining_bytes);
            hash_vector = compress(hash_vector, partial_vector);
        }

        finalize(hash_vector)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn all_blocks_are_consumed() {
        let mut bytes = [42u8; 1200];

        let ref_hash = gxhash(&bytes);

        for i in 0..bytes.len() {
            let swap = bytes[i];
            bytes[i] = 82;
            let new_hash = gxhash(&bytes);
            bytes[i] = swap;

            assert_ne!(ref_hash, new_hash, "byte {i} not processed");
        }
    }

    #[test]
    fn hash_of_zero_is_not_zero() {
        assert_ne!(0, gxhash(&[0u8; 0]));
        assert_ne!(0, gxhash(&[0u8; 1]));
        assert_ne!(0, gxhash(&[0u8; 1200]));
    }
}