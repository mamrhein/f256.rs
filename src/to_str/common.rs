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
