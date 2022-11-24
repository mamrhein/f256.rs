// ---------------------------------------------------------------------------
// Copyright:   (c) 2022 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

/// Returns ⌊log₁₀(2ⁱ)⌋ for 0 <= i <= 262144.
#[inline(always)]
pub(super) fn floor_log10_pow2(i: i32) -> i32 {
    debug_assert!(i >= 0);
    debug_assert!(i <= 262144);
    ((i as u128 * 169464822037455) >> 49) as i32
}
