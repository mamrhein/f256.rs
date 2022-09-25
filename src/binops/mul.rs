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
    ops::Mul,
};

use crate::{
    f256, u256, uint256::u256_mul, EMAX, EXP_BIAS, EXP_BITS, EXP_MAX,
    HI_ABS_MASK, HI_EXP_MASK, HI_FRACTION_BIAS, HI_FRACTION_BITS,
    HI_FRACTION_MASK, HI_SIGN_MASK, INF_HI, MAX_HI, SIGNIFICAND_BITS,
    TOTAL_BITS,
};

/// Compute z = x * y, rounded tie to even.
#[inline]
pub(crate) fn mul(x: f256, y: f256) -> f256 {
    // The products sign is the XOR of the signs of the operands.
    let z_hi_sign = (x.bits.hi ^ y.bits.hi) & HI_SIGN_MASK;

    // Check whether one or both operands are NaN, infinite or zero.
    // We mask off the sign bit and mark subnormals having a significand less
    // than 2¹²⁸ in least bit of the representations high u128. This allows to
    // use only that part for the handling of special cases.
    let x_abs_hi = (x.bits.hi & HI_ABS_MASK) | (x.bits.lo != 0) as u128;
    let y_abs_hi = (y.bits.hi & HI_ABS_MASK) | (y.bits.lo != 0) as u128;
    if max(x_abs_hi.wrapping_sub(1), y_abs_hi.wrapping_sub(1)) >= MAX_HI {
        let max_abs_hi = max(x_abs_hi, y_abs_hi);
        let min_abs_hi = min(x_abs_hi, y_abs_hi);
        if min_abs_hi == 0 {
            // Atleast one operand is zero.
            if max_abs_hi < HI_EXP_MASK {
                // ±0 × ±finite or ±finite × ±0
                return f256 {
                    bits: u256::new(z_hi_sign, 0),
                };
            };
            if max_abs_hi == HI_EXP_MASK {
                // ±0 × ±Inf or ±Inf × ±0
                return f256::NAN;
            }
        }
        if max_abs_hi > HI_EXP_MASK {
            // Atleast one operand is NAN.
            return f256::NAN;
        }
        // Atleast one operand is infinite and the other non-zero.
        return f256 {
            bits: u256::new(z_hi_sign | INF_HI, 0),
        };
    }

    // Both operands are finite and non-zero.
    let mut x_exp = x.biased_exponent() as i32;
    let mut x_signif = x.significand();
    let mut y_exp = y.biased_exponent() as i32;
    let mut y_signif = y.significand();

    // Check if operands are subnormal.
    if x_exp == 0 {
        if y_exp == 0 {
            // The product of two subnormals is zero.
            return f256 {
                bits: u256::new(z_hi_sign, 0),
            };
        } else {
            let sh = x_signif.leading_zeros() - EXP_BITS;
            x_signif <<= sh;
            x_exp -= sh as i32;
        }
    }
    if y_exp == 0 {
        let sh = y_signif.leading_zeros() - EXP_BITS;
        y_signif <<= sh;
        y_exp -= sh as i32;
    }

    // Calculate the results significand and exponent.
    // Shifting one operand to msb = 255 causes the result to its msb at
    // position 237 or 238. Normalizing it will atmost be a left-shift by 1.
    let (mut z_signif, rem) = u256_mul(&x_signif, &(y_signif << EXP_BITS));
    let mut z_exp = x_exp + y_exp - EXP_BIAS as i32;

    // Normalize result
    if z_signif.hi & HI_FRACTION_BIAS != 0 {
        z_exp += 1;
    } else {
        z_signif <<= 1;
    }

    // If the result overflows the range of values representable as `f256`,
    // return +/- Infinity.
    if z_exp >= EXP_MAX as i32 {
        return f256 {
            bits: u256::new(z_hi_sign | INF_HI, 0),
        };
    }

    // TODO: Check subnormal result.

    // Erase hidden bit and set exponent and sign.
    let mut bits = z_signif;
    bits.hi &= HI_FRACTION_MASK;
    bits.hi |= (z_exp as u128) << HI_FRACTION_BITS as u128;
    bits.hi |= z_hi_sign;

    // Final rounding. Possibly overflowing into the exponent, but that is ok.
    const TIE: u64 = 1_u64 << 63;
    if rem > TIE || (rem == TIE && ((bits.lo & 1) == 1)) {
        bits.incr();
    }
    f256 { bits }
}

impl Mul for f256 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        mul(self, rhs)
    }
}

forward_ref_binop!(impl Mul, mul);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mul_nan() {
        assert!((f256::NAN * f256::ONE).is_nan());
        assert!((f256::ONE * f256::NAN).is_nan());
        assert!((f256::NAN * f256::NAN).is_nan());
        assert!((f256::NAN * f256::INFINITY).is_nan());
        assert!((f256::INFINITY * f256::NAN).is_nan());
        assert!((f256::NAN * f256::NEG_INFINITY).is_nan());
        assert!((f256::NEG_INFINITY * f256::NAN).is_nan());
    }

    #[test]
    fn test_mul_inf() {
        assert_eq!(f256::INFINITY * f256::INFINITY, f256::INFINITY);
        assert_eq!(f256::INFINITY * f256::ONE, f256::INFINITY);
        assert_eq!(f256::ONE * f256::INFINITY, f256::INFINITY);
        assert_eq!(f256::NEG_INFINITY * f256::NEG_INFINITY, f256::INFINITY);
        assert_eq!(f256::NEG_INFINITY * f256::ONE, f256::NEG_INFINITY);
        assert_eq!(f256::ONE * f256::NEG_INFINITY, f256::NEG_INFINITY);
        assert_eq!((f256::INFINITY * f256::NEG_INFINITY), f256::NEG_INFINITY);
        assert_eq!((f256::NEG_INFINITY * f256::INFINITY), f256::NEG_INFINITY);
        assert!((f256::ZERO * f256::INFINITY).is_nan());
        assert!((f256::INFINITY * f256::ZERO).is_nan());
        assert!((f256::ZERO * f256::NEG_INFINITY).is_nan());
        assert!((f256::NEG_INFINITY * f256::ZERO).is_nan());
        assert!((f256::NEG_ZERO * f256::INFINITY).is_nan());
        assert!((f256::INFINITY * f256::NEG_ZERO).is_nan());
        assert!((f256::NEG_ZERO * f256::NEG_INFINITY).is_nan());
        assert!((f256::NEG_INFINITY * f256::NEG_ZERO).is_nan());
    }

    #[test]
    fn test_mul_zero() {
        assert_eq!(f256::ZERO * f256::ZERO, f256::ZERO);
        assert_eq!(f256::ZERO * f256::NEG_ZERO, f256::NEG_ZERO);
        assert_eq!(f256::NEG_ZERO * f256::ZERO, f256::NEG_ZERO);
        assert_eq!(f256::NEG_ZERO * f256::NEG_ZERO, f256::ZERO);
        assert_eq!(f256::ONE * f256::ZERO, f256::ZERO);
        assert_eq!(f256::ZERO * f256::ONE, f256::ZERO);
        assert_eq!(f256::ONE * f256::NEG_ZERO, f256::NEG_ZERO);
        assert_eq!(f256::NEG_ZERO * f256::ONE, f256::NEG_ZERO);
        assert_eq!(f256::NEG_ONE * f256::ZERO, f256::NEG_ZERO);
        assert_eq!(f256::ZERO * f256::NEG_ONE, f256::NEG_ZERO);
        assert_eq!(f256::NEG_ONE * f256::NEG_ZERO, f256::ZERO);
        assert_eq!(f256::NEG_ZERO * f256::NEG_ONE, f256::ZERO);
    }

    #[test]
    fn test_mul_normal() {
        assert_eq!(f256::ONE * f256::ONE, f256::ONE);
        assert_eq!(f256::ONE * f256::NEG_ONE, f256::NEG_ONE);
        assert_eq!(f256::NEG_ONE * f256::ONE, f256::NEG_ONE);
        assert_eq!(f256::TWO * f256::TWO, f256::from(4.0));
        assert_eq!(f256::from(3.5) * f256::from(2.75), f256::from(9.625));
    }

    #[test]
    fn test_overflow() {
        assert_eq!(f256::MAX * f256::TWO, f256::INFINITY);
        assert_eq!(f256::MIN * f256::TWO, f256::NEG_INFINITY);
    }
}
