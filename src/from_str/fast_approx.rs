// ---------------------------------------------------------------------------
// Copyright:   (c) 2022 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use crate::{f256, u256};

// We have a number with a decimal representation (-1)ˢ × w × 10ᵏ,
// where s ∈ {0, 1}, 0 < w < 2²⁵⁶ and |k| <= 78913.
// We need to transform it into (-1)ˢ × (1 + m × 2¹⁻ᵖ) × 2ᵉ,
// where p = 237, Eₘᵢₙ <= e - Eₘₐₓ <= Eₘₐₓ and 0 < m < 2ᵖ⁻¹.
//
// Under the conditions |k| <= 110 we apply the following:
// w × 10ᵏ = w × 5ᵏ × 2ᵏ.
// |k| <= 110 => 5ᵏ < 2²⁵⁶ => 5ᵏ is representable as a u256 value v.
// w' = w << l, where l = leading zeroes of w.
// Calculating w' × v / 2²⁵⁶ (if k >= 0) or as w / v / 2²⁵⁶ (if k < 0) gives a
// correctly rounded result (1 + m × 2¹⁻ᵖ) × 2ᵗ.
// Finally, setting e = t + k and setting the sign gives the required result.
pub(super) fn try_fast_approx(s: u32, w: u256, k: i32) -> Option<f256> {
    debug_assert!(s == 0 || s == 1);
    debug_assert!(!w.is_zero());
    None
}
