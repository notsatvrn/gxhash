pub mod soft;

#[cfg(not(any(all(nightly, target_arch = "arm"), target_arch = "aarch64", target_arch = "x86", target_arch = "x86_64")))]
use soft as platform;

#[cfg(any(all(nightly, target_arch = "arm"), target_arch = "aarch64"))]
#[path = "arm.rs"]
mod platform;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[path = "x86.rs"]
mod platform;

pub use platform::*;

pub trait AESHasherState {
    unsafe fn get_partial(p: *const Self, len: usize) -> Self;
}

pub fn new_optimal() {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
}

// 4KiB is the default page size for most systems, and conservative for other systems such as MacOS ARM (16KiB)
const PAGE_SIZE: usize = 0x1000;

#[inline]
unsafe fn check_same_page(ptr: *const State) -> bool {
    let address = ptr as usize;
    // Mask to keep only the last 12 bits
    let offset_within_page = address & (PAGE_SIZE - 1);
    // Check if the 16th byte from the current offset exceeds the page boundary
    offset_within_page < PAGE_SIZE - 16
}

pub(crate) const KEYS: [u32; 12] = 
   [0xF2784542, 0xB09D3E21, 0x89C222E5, 0xFC3BC28E,
    0x03FCE279, 0xCB6B2E9B, 0xB361DC58, 0x39132BD9,
    0xD0012E32, 0x689D2B7D, 0x5544B1B7, 0xC78B122B];

macro_rules! load_many_unaligned {
    ($ptr:ident, $($var:ident),+) => {
        $(
            #[allow(unused_mut)]
            let mut $var = load_unaligned($ptr);
            $ptr = ($ptr).offset(1);
        )+
    };
}

pub(crate) use load_many_unaligned;

#[inline(always)]
pub(crate) unsafe fn compress_all(input: &[u8]) -> State {

    let len = input.len();
    let mut ptr = input.as_ptr() as *const State;

    if len == 0 {
        return new();
    }

    if len <= 16 {
        // Input fits on a single SIMD vector, however we might read beyond the input message
        // Thus we need this safe method that checks if it can safely read beyond or must copy
        return get_partial(ptr, len);
    }

    let mut hash_vector: State;
    let end = ptr as usize + len;

    let extra_bytes_count = len % 16;
    if extra_bytes_count == 0 {
        load_many_unaligned!(ptr, v0);
        hash_vector = v0;
    } else {
        // If the input length does not match the length of a whole number of SIMD vectors,
        // it means we'll need to read a partial vector. We can start with the partial vector first,
        // so that we can safely read beyond since we expect the following bytes to still be part of
        // the input
        hash_vector = get_partial_unsafe(ptr, extra_bytes_count);
        ptr = ptr.cast::<u8>().add(extra_bytes_count).cast();
    }

    load_many_unaligned!(ptr, v0);

    if len > 32 {
        // Fast path when input length > 32 and <= 48
        load_many_unaligned!(ptr, v);
        v0 = aes_encrypt(v0, v);

        if len > 48 {
            // Fast path when input length > 48 and <= 64
            load_many_unaligned!(ptr, v);
            v0 = aes_encrypt(v0, v);

            if len > 64 {
                // Input message is large and we can use the high ILP loop
                hash_vector = compress_many(ptr, end, hash_vector, len);
            }
        }
    }
    
    return aes_encrypt_last(hash_vector, 
        aes_encrypt(aes_encrypt(v0, ld(KEYS.as_ptr())), ld(KEYS.as_ptr().offset(4))));
}

#[inline(always)]
unsafe fn compress_many(mut ptr: *const State, end: usize, hash_vector: State, len: usize) -> State {

    const UNROLL_FACTOR: usize = 8;

    let remaining_bytes = end -  ptr as usize;

    let unrollable_blocks_count: usize = remaining_bytes / (16 * UNROLL_FACTOR) * UNROLL_FACTOR; 

    let remaining_bytes = remaining_bytes - unrollable_blocks_count * 16;
    let end_address = ptr.add(remaining_bytes / 16) as usize;

    // Process first individual blocks until we have an whole number of 8 blocks
    let mut hash_vector = hash_vector;
    while (ptr as usize) < end_address {
        load_many_unaligned!(ptr, v0);
        hash_vector = aes_encrypt(hash_vector, v0);
    }

    // Process the remaining n * 8 blocks
    // This part may use 128-bit or 256-bit
    compress_8(ptr, end, hash_vector, len)
}

#[inline]
pub unsafe fn compress_8(mut ptr: *const State, end_address: usize, hash_vector: State, len: usize) -> State {
    #[cfg(all(target_arch = "x86_64", feature = "std", nightly))]
    if std::arch::is_x86_feature_detected!("vaes") {
        compress_8_hybrid(ptr, end_address, hash_vector, len);
    }

    // Disambiguation vectors
    let mut t1: State = new();
    let mut t2: State = new();

    // Hash is processed in two separate 128-bit parallel lanes
    // This allows the same processing to be applied using 256-bit V-AES instrinsics
    // so that hashes are stable in both cases. 
    let mut lane1 = hash_vector;
    let mut lane2 = hash_vector;

    while (ptr as usize) < end_address {

        crate::gxhash::load_unaligned!(ptr, v0, v1, v2, v3, v4, v5, v6, v7);

        v0 = aes_encrypt(v0, v2);
        v1 = aes_encrypt(v1, v3);

        v0 = aes_encrypt(v0, v4);
        v1 = aes_encrypt(v1, v5);

        v0 = aes_encrypt(v0, v6);
        v1 = aes_encrypt(v1, v7);

        t1 = add_u8(t1, ld(KEYS.as_ptr()));
        t2 = add_u8(t2, ld(KEYS.as_ptr().offset(4)));

        lane1 = aes_encrypt_last(aes_encrypt(v0, t1), lane1);
        lane2 = aes_encrypt_last(aes_encrypt(v1, t2), lane2);
    }
    // For 'Zeroes' test
    let len_vec = load_u32(len as u32);
    lane1 = add_u8(lane1, len_vec);
    lane2 = add_u8(lane2, len_vec);
    // Merge lanes
    aes_encrypt(lane1, lane2)
}

#[inline]
pub unsafe fn finalize(hash: State) -> State {
    let mut hash = aes_encrypt(hash, ld(KEYS.as_ptr()));
    hash = aes_encrypt(hash, ld(KEYS.as_ptr().offset(4)));
    hash = aes_encrypt_last(hash, ld(KEYS.as_ptr().offset(8)));

    hash
}
