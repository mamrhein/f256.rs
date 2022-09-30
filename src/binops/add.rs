// ---------------------------------------------------------------------------
// Copyright:   (c) 2022 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use core::{
    cmp::{max, min},
    ops::{Add, AddAssign, Sub, SubAssign},
};
use std::cmp::Ordering;

use crate::{
    f256, u256, EXP_MAX, HI_ABS_MASK, HI_EXP_MASK, HI_FRACTION_BIAS,
    HI_FRACTION_BITS, HI_FRACTION_MASK, HI_SIGN_MASK, INF_HI, MAX_HI,
    SIGNIFICAND_BITS,
};

#[inline]
pub(crate) fn add(x: f256, y: f256) -> f256 {
    // Check whether one or both operands are NaN, infinite or zero.
    // We mask off the sign bit and mark subnormals having a significand less
    // than 2¹²⁸ in least bit of the representations high u128. This allows to
    // use only that part for the handling of special cases.
    let x_abs_hi = (x.bits.hi & HI_ABS_MASK) | (x.bits.lo != 0) as u128;
    let y_abs_hi = (y.bits.hi & HI_ABS_MASK) | (y.bits.lo != 0) as u128;
    if max(x_abs_hi.wrapping_sub(1), y_abs_hi.wrapping_sub(1)) >= MAX_HI {
        let x_sign = x.bits.hi & HI_SIGN_MASK;
        let y_sign = y.bits.hi & HI_SIGN_MASK;
        let max_abs_hi = max(x_abs_hi, y_abs_hi);
        if max_abs_hi == 0 {
            // Both operands are zero.
            return f256 {
                bits: u256::new(x_sign & y_sign, 0),
            };
        }
        if max_abs_hi > HI_EXP_MASK
            || (x_abs_hi == INF_HI && y_abs_hi == INF_HI && x_sign != y_sign)
        {
            // Atleast one operand is NAN, or both operands are infinite and
            // their signs differ.
            return f256::NAN;
        }
        // For all other special cases return the operand with the greater
        // absolute value.
        return if x_abs_hi > y_abs_hi { x } else { y };
    }

    // Both operands are finite and non-zero.
    // Compare the absolute values of the operands and swap them in case
    // |x| < |y|.
    let mut a: f256 = x.abs();
    let mut b: f256 = y.abs();
    if a >= b {
        a = x;
        b = y;
    } else {
        a = y;
        b = x;
    }

    // The sign of the result is the sign of the operand with the greater
    // absolute value.
    let hi_sign = a.bits.hi & HI_SIGN_MASK;

    // Extract biased exponents and significands (shifted left by 3 bits to give
    // room for a round, guard and sticky bit). These shifts are safe because
    // the significands use at most 237 bits in an u256.
    let mut a_exp = a.biased_exponent();
    let b_exp = b.biased_exponent();
    let mut a_signif = a.significand() << 3;
    let mut b_signif = b.significand() << 3;

    // Here a >= b => a_exp >= b_exp => a_exp - b_exp >= 0.
    // We adjust the significand of b by right-shifting it.
    // We limit the adjustment by an upper limit of SIGNIFICAND_BITS + 2. Thus,
    // the silent bit of b's significant is atmost to the position of the sticky
    // bit. Any further shift would have no effect on the result.
    let adj = min(a_exp - b_exp, SIGNIFICAND_BITS + 2);
    let sticky_bit = !(adj == 0 || (b_signif << (u256::BITS - adj)).is_zero());
    b_signif >>= adj;
    b_signif.lo |= sticky_bit as u128;

    // Determine the actual op to be performed: if the sign of the operands
    // differ, it's a subtraction, otherwise an addition.
    if ((x.bits.hi ^ y.bits.hi) & HI_SIGN_MASK) == 0 {
        a_signif += &b_signif;
        // If addition carried over, right-shift the significand and increment
        // the exponent.
        if (a_signif.hi >> (HI_FRACTION_BITS + 4)) != 0 {
            a_signif >>= 1;
            a_signif.lo |= sticky_bit as u128;
            a_exp += 1;
        }
    } else {
        a_signif -= &b_signif;
        if a_signif.is_zero() {
            return f256::ZERO;
        }
        // If subtraction cancelled the hidden bit, left-shift the significand
        // and decrement the exponent.
        if a_signif.hi < HI_FRACTION_BIAS << 3 {
            let adj = min(SIGNIFICAND_BITS + 2 - a_signif.msb(), a_exp);
            a_signif <<= adj;
            a_exp -= adj;
        }
    }

    // If the result overflows the range of values representable as `f256`,
    // return ±Inf.
    if a_exp >= EXP_MAX {
        return f256 {
            bits: u256::new(INF_HI | hi_sign, 0),
        };
    }

    // Get round, guard and sticky bit.
    let l3bits = (a_signif.lo & 0x7_u128) as u32;
    // Shift significand back, erase hidden bit and set exponent and sign.
    let mut bits = a_signif >> 3;
    bits.hi &= HI_FRACTION_MASK;
    bits.hi |= (a_exp as u128) << HI_FRACTION_BITS;
    bits.hi |= hi_sign;
    // Final rounding. Possibly overflowing into the exponent, but that is ok.
    if l3bits > 0x4 || l3bits == 0x4 && (bits.lo & 1) == 1 {
        bits.incr();
    }
    f256 { bits }
}

impl Add for f256 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        add(self, rhs)
    }
}

forward_ref_binop!(impl Add, add);

forward_op_assign!(impl AddAssign, add_assign, Add, add);

impl Sub for f256 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        add(self, -rhs)
    }
}

forward_ref_binop!(impl Sub, sub);

forward_op_assign!(impl SubAssign, sub_assign, Sub, sub);
