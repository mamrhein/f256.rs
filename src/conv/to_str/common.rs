// ---------------------------------------------------------------------------
// Copyright:   (c) 2022 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use core::f64::consts::LOG10_2;

use crate::U256;

/// Returns ⌊log₁₀(2ⁱ)⌋.
#[inline(always)]
#[allow(clippy::cast_possible_truncation)]
pub(crate) const fn floor_log10_pow2(i: i32) -> i32 {
    ((i as i128 * 169464822037455) >> 49) as i32
}

/// Returns ⌊log₁₀(m × 2ⁱ)⌋.
#[inline(always)]
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_precision_loss)]
pub(crate) fn floor_log10f(m: U256, i: i32) -> i32 {
    (((m.hi.0 as f64).log2() + 128_f64 + (i as f64)) * LOG10_2).trunc() as i32
}
