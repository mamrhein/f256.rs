// ---------------------------------------------------------------------------
// Copyright:   (c) 2023 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use super::sos::sum_squares;
use crate::big_uint::{BigUInt, HiLo};
use crate::math::sqrt::square_root;
use crate::{
    abs_bits, abs_bits_sticky, f256, signif, BinEncAnySpecial, EMAX, EMIN,
    EXP_BIAS, EXP_MAX, FRACTION_BITS, HI_EXP_MASK, HI_FRACTION_BITS,
};

#[allow(clippy::cast_possible_wrap)]
#[allow(clippy::cast_sign_loss)]
pub(crate) fn hypot(x: &f256, y: &f256) -> f256 {
    // Squares are always positive, so there's no need to care about the signs
    // of the operands.
    let mut abs_bits_x = abs_bits(x);
    let mut abs_bits_y = abs_bits(y);
    // Check whether one or both operands are NaN, infinite or zero.
    // We mask off the sign bit and mark subnormals having a significand less
    // than 2¹²⁸ in least bit of the representations high u128. This allows to
    // use only that part for the handling of special cases.
    let abs_bits_sticky_x = abs_bits_sticky(&abs_bits_x);
    let abs_bits_sticky_y = abs_bits_sticky(&abs_bits_y);
    if (abs_bits_sticky_x, abs_bits_sticky_y).any_special() {
        if abs_bits_sticky_x > HI_EXP_MASK || abs_bits_sticky_y > HI_EXP_MASK
        {
            // Atleast one operand is NAN.
            return f256::NAN;
        }
        if abs_bits_sticky_x == 0 {
            // x is zero.
            return f256 { bits: abs_bits_y };
        }
        if abs_bits_sticky_y == 0 {
            // y is zero.
            return f256 { bits: abs_bits_x };
        }
        // For all other special cases the result is INF.
        return f256::INFINITY;
    }

    // Both operands are finite and non-zero.

    // Calculate √(x² + y²).
    let (signif, exp) = sum_squares(&mut abs_bits_x, &mut abs_bits_y);
    let (mut p, mut q) = square_root(&signif.hi, exp);
    if p > EMAX {
        return f256::INFINITY;
    }
    if p < EMIN {
        // Result is subnormal
        let shr = (EMIN - p + 1) as u32;
        return f256 {
            bits: q.rounding_div_pow2(shr),
        };
    }
    // Final reconstruction and rounding.
    let exp_bits_minus_1 = (EXP_BIAS as i32 + p - 1) as u128;
    // The sqare root of a floating point number can't be an exact
    // midpoint between two consecutive floating point numbers, so there
    // is no need to care about ties.
    let r = q.lo_t().lo_t() & 1;
    let mut bits = q >> 1;
    debug_assert_eq!(bits.hi.msb(), HI_FRACTION_BITS);
    bits.hi.0 += exp_bits_minus_1 << HI_FRACTION_BITS;
    // Final rounding. Possibly overflowing into the exponent, but that is ok.
    bits.incr_if(r == 1);
    f256 { bits }
}
