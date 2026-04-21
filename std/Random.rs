/* 
 * (C) 2026 Bogdan Yachmenev
 * Licensed under MIT License (https://mit-license.org)
 * This file is part of the [Random] Standard Library.
 */

//#ARROW_IGNORE
use std::time::{SystemTime, UNIX_EPOCH};
use std::mem::MaybeUninit;
use std::hint::black_box;

pub fn random_int(min: u64, max: u64) -> u64 {
    if min >= max { return min; }

    // 1. Get entropy from System Time (nanos)
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;

    // 2. Extract "entropy" from uninitialized stack memory.
    // black_box prevents the compiler from optimizing out this "useless" read.
    let [seed, salt]: [u64; 2] = unsafe {
        let mut data = MaybeUninit::<[u64; 2]>::uninit();
        black_box(data.assume_init())
    };

    // 3. Mix entropy using SplitMix64-based algorithm
    let mut x = nanos.wrapping_add(seed);
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
    let raw_random = x ^ (x >> 31) ^ salt;

    // 4. Bound to range
    let range = max - min + 1;
    min + (raw_random % range)
}

pub fn random_char() -> char {
    // 1. Accumulate entropy: time + stack garbage + memory address
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;
    
    // black_box ensures the compiler doesn't assume seed/salt are 0
    let [seed, salt]: [u64; 2] = unsafe { 
        black_box(MaybeUninit::uninit().assume_init()) 
    };
    
    let stack_ptr = black_box(&nanos as *const u64 as u64);

    // 2. Scramble bits
    let mut x = nanos.wrapping_add(seed).wrapping_add(stack_ptr);
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
    let raw_val = x ^ (x >> 31) ^ salt;

    // 3. Map to valid UTF-32 (skipping surrogates D800-DFFF)
    let unicode_range = 0x110000 - 0x800;
    let mut code_point = (raw_val % unicode_range) as u32;

    if code_point >= 0xD800 {
        code_point += 0x800;
    }

    char::from_u32(code_point).unwrap_or('')
}
//#ARROW_NO_IGNORE

