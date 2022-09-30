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
    ops::{Div, DivAssign},
};

use crate::{
    f256, u256, EXP_BIAS, EXP_BITS, EXP_MAX, HI_ABS_MASK, HI_EXP_MASK,
    HI_FRACTION_BIAS, HI_FRACTION_BITS, HI_FRACTION_MASK, HI_SIGN_MASK, INF_HI,
    MAX_HI, SIGNIFICAND_BITS,
};

// Calculate q = x' / y (rounded tie to even), where x' = x * 2²³⁷.
#[inline]
fn u256_div(x: &u256, y: &u256) -> u256 {
    debug_assert!(!y.is_zero());
    debug_assert!(x >= y);
    let mut r = *x;
    let mut q = u256::new(0, 1);
    r -= y;
    for i in 1..=SIGNIFICAND_BITS {
        let mut t = r << 1;
        t -= y;
        q <<= 1;
        r = t;
        if t > *y {
            r += y;
        } else {
            q.incr();
        }
    }
    let c = q.lo & 1;
    q >>= 1;
    q += c;
    q
}

// Compute z = x / y, rounded tie to even.
#[allow(clippy::cast_possible_wrap)]
#[allow(clippy::cast_sign_loss)]
#[inline]
pub(crate) fn div(x: f256, y: f256) -> f256 {
    // The products sign is the XOR of the signs of the operands.
    let hi_sign = (x.bits.hi ^ y.bits.hi) & HI_SIGN_MASK;

    // Check whether one or both operands are NaN, infinite or zero.
    // We mask off the sign bit and mark subnormals having a significand less
    // than 2¹²⁸ in least bit of the representations high u128. This allows to
    // use only that part for the handling of special cases.
    let x_abs_hi = (x.bits.hi & HI_ABS_MASK) | (x.bits.lo != 0) as u128;
    let y_abs_hi = (y.bits.hi & HI_ABS_MASK) | (y.bits.lo != 0) as u128;
    if max(x_abs_hi.wrapping_sub(1), y_abs_hi.wrapping_sub(1)) >= MAX_HI {
        let max_abs_hi = max(x_abs_hi, y_abs_hi);
        if max_abs_hi > HI_EXP_MASK || x_abs_hi == y_abs_hi {
            // Atleast one operand is NAN or we have ±0 / ±0 or ±Inf / ±Inf.
            return f256::NAN;
        }
        if x_abs_hi < y_abs_hi {
            // ±0 / ±Inf or ±0 / ±finite or ±finite / ±Inf.
            return f256 {
                bits: u256::new(hi_sign, 0),
            };
        }
        // ±Inf / ±0 or ±finite / ±0 or ±Inf / ±finite.
        return f256 {
            bits: u256::new(hi_sign | INF_HI, 0),
        };
    }

    // Both operands are finite and non-zero.
    let mut x_exp = x.biased_exponent() as i32;
    let mut x_signif = x.significand();
    let x_shift = x_signif.leading_zeros() - EXP_BITS;
    x_signif <<= x_shift;
    x_exp -= x_shift as i32 - (x_exp == 0) as i32;

    let mut y_exp = y.biased_exponent() as i32;
    let mut y_signif = y.significand();
    let y_shift = y_signif.leading_zeros() - EXP_BITS;
    y_signif <<= y_shift;
    y_exp -= y_shift as i32 - (y_exp == 0) as i32;

    // Assure x_signif >= y_signif.
    let c = (x_signif < y_signif) as u32;
    x_signif <<= c;

    // Calculate the results significand and exponent.
    let mut bits = u256_div(&x_signif, &y_signif);
    let mut exp = x_exp - y_exp + (EXP_BIAS - c) as i32;

    // If the result overflows the range of values representable as `f256`,
    // return +/- Infinity.
    if exp >= EXP_MAX as i32 {
        return f256 {
            bits: u256::new(hi_sign | INF_HI, 0),
        };
    }

    // Assemble the result.
    if exp <= 0 {
        let shift = (1 - exp) as u32;
        if shift > bits.msb() {
            // Result underflows to zero.
            return f256 {
                bits: u256::new(hi_sign, 0),
            };
        }
        // Adjust the remainder for correct final rounding.
        let rem = ((bits << (u256::BITS - shift)).hi >> 64) as u64;
        bits >>= shift;
        const TIE: u64 = 1_u64 << 63;
        if rem > TIE || (rem == TIE && ((bits.lo & 1) == 1)) {
            bits.incr();
        }
    } else {
        // Erase hidden bit and set exponent.
        bits.hi &= HI_FRACTION_MASK;
        bits.hi |= (exp as u128) << HI_FRACTION_BITS as u128;
    }
    bits.hi |= hi_sign;
    f256 { bits }
}

impl Div for f256 {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        div(self, rhs)
    }
}

forward_ref_binop!(impl Div, div);

forward_op_assign!(impl DivAssign, div_assign, Div, div);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_div_nan() {
        assert!((f256::NAN / f256::ONE).is_nan());
        assert!((f256::ONE / f256::NAN).is_nan());
        assert!((f256::NAN / f256::NAN).is_nan());
        assert!((f256::NAN / f256::INFINITY).is_nan());
        assert!((f256::INFINITY / f256::NAN).is_nan());
        assert!((f256::NAN / f256::NEG_INFINITY).is_nan());
        assert!((f256::NEG_INFINITY / f256::NAN).is_nan());
    }

    #[test]
    fn test_div_inf() {
        assert_eq!(f256::INFINITY / f256::ONE, f256::INFINITY);
        assert_eq!(f256::ONE / f256::INFINITY, f256::ZERO);
        assert_eq!(f256::NEG_INFINITY / f256::ONE, f256::NEG_INFINITY);
        assert_eq!(f256::ONE / f256::NEG_INFINITY, f256::NEG_ZERO);
        assert!((f256::INFINITY / f256::INFINITY).is_nan());
        assert!((f256::INFINITY / f256::NEG_INFINITY).is_nan());
        assert!((f256::NEG_INFINITY / f256::INFINITY).is_nan());
        assert!((f256::NEG_INFINITY / f256::NEG_INFINITY).is_nan());
    }

    #[test]
    fn test_div_zero() {
        assert_eq!(f256::ONE / f256::ZERO, f256::INFINITY);
        assert_eq!(f256::ZERO / f256::ONE, f256::ZERO);
        assert_eq!(f256::ONE / f256::NEG_ZERO, f256::NEG_INFINITY);
        assert_eq!(f256::NEG_ZERO / f256::ONE, f256::NEG_ZERO);
        assert_eq!(f256::NEG_ONE / f256::ZERO, f256::NEG_INFINITY);
        assert_eq!(f256::ZERO / f256::NEG_ONE, f256::NEG_ZERO);
        assert_eq!(f256::NEG_ONE / f256::NEG_ZERO, f256::INFINITY);
        assert_eq!(f256::NEG_ZERO / f256::NEG_ONE, f256::ZERO);
        assert!((f256::ZERO / f256::ZERO).is_nan());
        assert!((f256::ZERO / f256::NEG_ZERO).is_nan());
        assert!((f256::NEG_ZERO / f256::ZERO).is_nan());
        assert!((f256::NEG_ZERO / f256::NEG_ZERO).is_nan());
    }

    #[test]
    fn test_div_normal() {
        assert_eq!(f256::ONE / f256::ONE, f256::ONE);
        assert_eq!(f256::ONE / f256::NEG_ONE, f256::NEG_ONE);
        assert_eq!(f256::NEG_ONE / f256::ONE, f256::NEG_ONE);
        assert_eq!(f256::from(4.0) / f256::TWO, f256::TWO);
        assert_eq!(f256::from(9.625) / f256::from(2.75), f256::from(3.5));
    }

    #[test]
    fn test_div_subnormal() {
        let x = f256::MIN_GT_ZERO;
        assert_eq!(x / x, f256::ONE);
        assert_eq!(-x / x, f256::NEG_ONE);
        assert_eq!(x / -x, f256::NEG_ONE);
        assert_eq!(-x / -x, f256::ONE);
        let y = f256::TWO;
        assert_eq!(x / y, f256::ZERO);
        assert_eq!(-x / y, f256::NEG_ZERO);
        assert_eq!(x / -y, f256::NEG_ZERO);
        assert_eq!(-x / -y, f256::ZERO);
        let y = f256::from(0.5);
        let z = x + x;
        assert_eq!(x / y, z);
    }

    #[test]
    fn test_overflow() {
        assert_eq!(f256::MAX / f256::MIN_GT_ZERO, f256::INFINITY);
        assert_eq!(f256::MIN / f256::MIN_GT_ZERO, f256::NEG_INFINITY);
    }
}
