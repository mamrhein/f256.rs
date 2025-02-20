// ---------------------------------------------------------------------------
// Copyright:   (c) 2023 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use core::{
    cmp::{max, min},
    ops::{Rem, RemAssign},
};

use crate::{
    abs_bits, abs_bits_sticky, exp_bits, f256, norm_bit, sign_bits_hi,
    signif, BigUInt, BinEncAnySpecial, DivRem, HiLo, FRACTION_BITS,
    HI_EXP_MASK, HI_FRACTION_BITS, MAX_HI, SIGNIFICAND_BITS, U256, U512,
};

// Compute z = x % y.
#[inline]
pub(crate) fn rem(x: f256, y: f256) -> f256 {
    let mut abs_bits_x = abs_bits(&x);
    let mut abs_bits_y = abs_bits(&y);
    // Check whether one or both operands are NaN, infinite or zero.
    // We mask off the sign bit and mark subnormals having a significand less
    // than 2¹²⁸ in least bit of the representations high u128. This allows to
    // use only that part for the handling of special cases.
    let abs_bits_sticky_x = abs_bits_sticky(&abs_bits_x);
    let abs_bits_sticky_y = abs_bits_sticky(&abs_bits_y);
    if (abs_bits_sticky_x, abs_bits_sticky_y).any_special() {
        let max_abs_bits_sticky = max(abs_bits_sticky_x, abs_bits_sticky_y);
        if max_abs_bits_sticky > HI_EXP_MASK
            || abs_bits_sticky_y == 0
            || abs_bits_sticky_x == HI_EXP_MASK
        {
            // Atleast one operand is NAN or we have x % ±0 or ±Inf % y.
            return f256::NAN;
        }
        // x % ±Inf for x ∈ {±0, ±finite} = x, and
        // ±0 % ±finite = ±0.
        return x;
    }

    // Both operands are finite and non-zero.
    // x % y = x - q * y, where q = ⌊x / y⌋
    // x % y = s * (|x| - q * |y|), where
    //    q = ⌊|x| / |y|⌋ and
    //    s = 1 if x >= 0 and -1 otherwise

    // |x| < |y| => x % y = x
    if abs_bits_x < abs_bits_y {
        return x;
    }

    // Extract biased exponents and significands.
    let mut exp_bits_x = exp_bits(&abs_bits_x);
    let norm_bit_x = norm_bit(&abs_bits_x);
    let mut signif_x = signif(&abs_bits_x);
    let mut exp_bits_y = exp_bits(&abs_bits_y);
    let norm_bit_y = norm_bit(&abs_bits_y);
    let mut signif_y = signif(&abs_bits_y);

    // If the significands are equal, |x| is an integral multiple of |y|.
    if signif_x == signif_y {
        return f256::ZERO;
    }

    let n_bits = exp_bits_x + norm_bit_y - exp_bits_y - norm_bit_x;
    let sh = n_bits % U256::BITS;
    let mut t = U512::from_hi_lo(U256::ZERO, signif_x);
    t <<= sh;
    let mut abs_bits_z = &t % &signif_y;
    for _ in 0..n_bits >> 8 {
        t = U512::from_hi_lo(abs_bits_z, U256::ZERO);
        abs_bits_z = &t % &signif_y;
        if abs_bits_z.is_zero() {
            break;
        }
    }
    if abs_bits_z.is_zero() {
        return f256::ZERO;
    }
    let shift_z = min(
        FRACTION_BITS - abs_bits_z.msb(),
        exp_bits_y.saturating_sub(1),
    );
    abs_bits_z <<= shift_z;
    let exp_bits_z_m1 = (exp_bits_y - shift_z).saturating_sub(1);
    abs_bits_z.hi.0 += (exp_bits_z_m1 as u128) << HI_FRACTION_BITS;
    f256 {
        bits: U256::new(sign_bits_hi(&x) | abs_bits_z.hi.0, abs_bits_z.lo.0),
    }
}

impl Rem for f256 {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self::Output {
        rem(self, rhs)
    }
}

forward_ref_binop!(impl Rem, rem);

forward_op_assign!(impl RemAssign, rem_assign, Rem, rem);
