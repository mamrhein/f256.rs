// ---------------------------------------------------------------------------
// Copyright:   (c) 2022 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

/// Implementation of a fast decimal to float conversion algorithm as
/// described in `Daniel Lemire: Number Parsing at a Gigabyte per
/// Second`, available at [https://arxiv.org/abs/2101.11408.pdf], adopted for `f256`.
use core::cmp::Ordering;

use super::{
    fast_exact::fast_exact,
    powers_of_five::{get_power_of_five, MAX_ABS_EXP},
    slow_exact::f256_exact,
};
use crate::{
    f256, BigUInt, EMAX, EXP_BIAS, EXP_BITS, HI_FRACTION_BIAS, U256,
};

#[allow(clippy::cast_possible_wrap)]
#[allow(clippy::cast_sign_loss)]
pub(super) fn fast_approx(
    lit: &str,
    sign: u32,
    exp10: i32,
    mut signif10: U256,
    signif_truncated: bool,
) -> f256 {
    // We have a number with a decimal representation (-1)ˢ × w × 10ᵏ,
    // where s ∈ {0, 1}, 0 < w < 2²⁵⁶ and |k| <= 78913.
    // We need to transform it into (-1)ˢ × (1 + m × 2¹⁻ᵖ) × 2ᵉ,
    // where p = 237, Eₘᵢₙ <= e <= Eₘₐₓ and 0 < m < 2ᵖ⁻¹.

    // w × 10ᵏ = w × 5ᵏ × 2ᵏ.
    // Under the conditions |k| <= 512 we approximate w × 5ᵏ by multiplying
    // the normalized significand w' (= w * 2²⁵⁶⁻ⁿˡᶻ where nlz is the
    // number of leading zeroes of w) and a pre-computed approximation of
    // 5ᵏ from a table T[k].
    let exp10_abs = exp10.unsigned_abs();
    if exp10_abs <= MAX_ABS_EXP {
        // Adjust significand to be in range [2²⁵⁵..2²⁵⁶).
        let signif10_nlz = signif10.leading_zeros();
        signif10 <<= signif10_nlz;
        // Compute w' * T[k]
        let (p5hi, p5lo, mut exp2) = get_power_of_five(exp10);
        let mut signif2 = signif10.truncating_mul(&U256::new(p5hi, p5lo));
        // As both multiplicands have their highest bit set, the result has
        // atmost one leading zero. We have to shift the highest bit to the
        // position of the hidden bit, i.e. by EXP_BITS - number of leading
        // zeroes and round the significand correctly.
        let signif2_nlz = signif2.leading_zeros();
        let mut shift = EXP_BITS - signif2_nlz;
        let mask = (1_u128 << shift) - 1;
        let tie = 1_u128 << (shift - 1);
        let fract = signif2.lo.0 & mask;
        signif2 >>= shift;
        // Check rounding condition.
        let round_up = match fract.cmp(&tie) {
            Ordering::Greater => {
                // Round up.
                Some(true)
            }
            Ordering::Less => {
                // Check edge case where truncated digits might cause
                // overflow.
                if fract == tie - 1 {
                    // Fall back to slow algorithm.
                    None
                } else {
                    // Round down.
                    Some(false)
                }
            }
            Ordering::Equal => {
                // Check edge cases.
                // If 5ᵏ ∈ [1..2²⁵⁶) we can be sure to have a tie, otherwise
                // we have "a tie and a little bit more".
                if (0..=110).contains(&exp10) {
                    Some(signif2.lo.is_odd())
                } else {
                    Some(true)
                }
            }
        };
        if let Some(up) = round_up {
            if up {
                signif2.incr();
                if signif2.hi.0 >= (HI_FRACTION_BIAS << 1) {
                    // Rounding overflowed, need to shift back.
                    signif2 >>= 1;
                    exp2 += 1;
                }
            }
            exp2 += 256 - signif10_nlz as i32 - signif2_nlz as i32 + exp10;
            if exp2 > EMAX {
                return [f256::INFINITY, f256::NEG_INFINITY][sign as usize];
            }
            let f = f256::new(sign, exp2, signif2);
            if signif_truncated {
                // The real significand w' has been truncated, so f may be
                // less than the correctly rounded result f'.
                // But w < w' < w+1 => f <= f' <= f"
                // where f" is the transformation of (-1)ˢ × (w+1) × 10ᵏ.
                // If f = f" then f = f'.
                let mut signif10_incr = signif10;
                signif10_incr.incr();
                if f == fast_approx(lit, sign, exp10, signif10_incr, false) {
                    return f;
                }
            } else {
                return f;
            }
        }
    }

    // The last resort must always succeed!
    f256_exact(lit)
}
