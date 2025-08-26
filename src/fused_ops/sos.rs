// ---------------------------------------------------------------------------
// Copyright:   (c) 2023 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use crate::big_uint::{UInt, U128};
use crate::{
    abs_bits, abs_bits_sticky, exp, f256, norm_bit, norm_signif_exp, signif,
    BigUInt, BinEncAnySpecial, HiLo, EMAX, EMIN, EXP_BIAS, EXP_BITS, EXP_MAX,
    FRACTION_BITS, HI_EXP_MASK, HI_FRACTION_BITS, HI_FRACTION_MASK, INF_HI,
    U256, U512,
};
use core::{
    cmp::max,
    mem::swap,
    ops::{ShlAssign, ShrAssign},
};

#[allow(clippy::cast_possible_wrap)]
#[allow(clippy::cast_sign_loss)]
pub(crate) fn sum_squares(
    abs_bits_x: &mut U256,
    abs_bits_y: &mut U256,
) -> (U512, i32) {
    // Compare the absolute values of the operands and swap them in case
    // |x| < |y|.
    if abs_bits_x < abs_bits_y {
        swap(abs_bits_x, abs_bits_y);
    }
    // Extract biased exponents and significands.
    let (mut signif_x, exp_x) = norm_signif_exp(&abs_bits_x);
    let (signif_y, exp_y) = norm_signif_exp(&abs_bits_y);

    // |x| >= |y| => x² >= y²
    // Square the operands significands. Shift the greater one left by 20 bits
    // to align its radix point to bit 512 - EXP_BITS - 1 = 492. This shift is
    // safe because the squares significands use at most 474 bits in an u512.
    // Adjust the significand of the smaller square according to the
    // difference of the squares exponents, setting a sticky bit in case of a
    // right shift.
    signif_x <<= 10;
    let t = signif_x.widening_mul(&signif_x);
    let mut signif_z = U512 { hi: t.1, lo: t.0 };
    let t = signif_y.widening_mul(&signif_y);
    let mut signif_y2 = U512 { hi: t.1, lo: t.0 };
    // |x| >= |y| => exp(x) >= exp(y), so the following can not overflow.
    let d = 2 * (exp_x - exp_y) as u32;
    match d {
        0..=19 => {
            signif_y2 <<= (20 - d);
        }
        20 => {}
        21..=494 => {
            let mut t = U512::default();
            let shr = d - 20;
            (signif_y2, t) = signif_y2.widening_shr(shr);
            signif_y2.lo.lo.0 |= (!t.is_zero()) as u128;
        }
        _ => {
            signif_y2 = U512::ONE;
        }
    }
    signif_z += &signif_y2;
    // The results radix point is aligned at bit 492 of signif_z, it has
    // atmost 3 leading bits, 2 from the squaring and 1 from the possible
    // overflow of the addition.
    debug_assert!(signif_z.leading_zeros() <= EXP_BITS);
    (signif_z, 2 * exp_x)
}

#[allow(clippy::cast_possible_wrap)]
#[allow(clippy::cast_sign_loss)]
pub(crate) fn sos(x: &f256, y: &f256) -> f256 {
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
    if (abs_bits_sticky_x, abs_bits_sticky_y).any_non_normal() {
        let max_abs_bits_sticky = max(abs_bits_sticky_x, abs_bits_sticky_y);
        if max_abs_bits_sticky <= HI_FRACTION_MASK {
            // Both operands are zero or subnormal.
            return f256::ZERO;
        }
        if max_abs_bits_sticky > HI_EXP_MASK {
            // Atleast one operand is NAN.
            return f256::NAN;
        }
        if abs_bits_sticky_x <= HI_FRACTION_MASK {
            // x is zero or subnormal.
            return y.square();
        }
        if abs_bits_sticky_y <= HI_FRACTION_MASK {
            // y is zero or subnormal.
            return x.square();
        }
        // For all other special cases the result is INF.
        return f256::INFINITY;
    }

    // Both operands are finite and non-zero.

    // Calculate x² + y².
    let (mut signif_z, mut exp_z) =
        sum_squares(&mut abs_bits_x, &mut abs_bits_y);

    // Convert intermediate result to f256.
    let shr = EXP_BITS - signif_z.hi.hi.0.leading_zeros();
    let sticky_bit = (signif_z.lo.lo.0 != 0) as u128;
    signif_z >>= shr;
    signif_z.lo.lo.0 |= sticky_bit;
    exp_z += shr as i32;
    if exp_z > EMAX {
        return f256::INFINITY;
    }
    let exp_bits_z_minus_1 = (EXP_BIAS as i32 + exp_z - 1) as u128;
    let rnd_bits = (signif_z.lo.hi.0 >> (u128::BITS - 3)) as u32
        | ((signif_z.lo.hi.0 << 3) != 0) as u32
        | (signif_z.lo.lo.0 != 0) as u32;
    let mut abs_bits_z = signif_z.hi;
    abs_bits_z.hi.0 += exp_bits_z_minus_1 << HI_FRACTION_BITS;
    // Final rounding. Possibly overflowing into the exponent, but that is ok.
    if rnd_bits > 0x4 || rnd_bits == 0x4 && abs_bits_z.lo.is_odd() {
        abs_bits_z.incr();
    }
    f256 { bits: abs_bits_z }
}
