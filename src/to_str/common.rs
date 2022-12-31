// ---------------------------------------------------------------------------
// Copyright:   (c) 2022 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

/// Returns ⌊log₁₀(2ⁱ)⌋.
#[inline(always)]
pub(super) fn floor_log10_pow2(i: i32) -> i32 {
    ((i as i128 * 169464822037455) >> 49) as i32
}

// The following code is copied from rust stdlib.
// TODO: replace it when Integer::{ilog,ilog2,ilog10} got stable.

// 0 < val < 100_000
#[inline]
const fn less_than_5(val: u32) -> u32 {
    // Similar to u8, when adding one of these constants to val,
    // we get two possible bit patterns above the low 17 bits,
    // depending on whether val is below or above the threshold.
    const C1: u32 = 0b011_00000000000000000 - 10; // 393206
    const C2: u32 = 0b100_00000000000000000 - 100; // 524188
    const C3: u32 = 0b111_00000000000000000 - 1000; // 916504
    const C4: u32 = 0b100_00000000000000000 - 10000; // 514288

    // Value of top bits:
    //                +c1  +c2  1&2  +c3  +c4  3&4   ^
    //         0..=9  010  011  010  110  011  010  000 = 0
    //       10..=99  011  011  011  110  011  010  001 = 1
    //     100..=999  011  100  000  110  011  010  010 = 2
    //   1000..=9999  011  100  000  111  011  011  011 = 3
    // 10000..=99999  011  100  000  111  100  100  100 = 4
    (((val + C1) & (val + C2)) ^ ((val + C3) & (val + C4))) >> 17
}

/// Returns ⌊log₁₀(n)⌋.
#[inline]
pub(super) fn floor_log10(mut n: u64) -> u32 {
    let mut log = 0;
    if n >= 10_000_000_000 {
        n /= 10_000_000_000;
        log += 10;
    }
    if n >= 100_000 {
        n /= 100_000;
        log += 5;
    }
    log + less_than_5(n as u32)
}
