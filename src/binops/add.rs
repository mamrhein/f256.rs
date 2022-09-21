// ---------------------------------------------------------------------------
// Copyright:   (c) 2022 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use core::ops::{Add, Sub};
use std::cmp::min;

use crate::{
    f256, u256, EXP_MAX, HI_FRACTION_BITS, HI_FRACTION_MASK, HI_SIGN_MASK,
    SIGNIFICAND_BITS,
};

#[inline]
pub(crate) fn add_special(x: f256, y: f256) -> f256 {
    // Either x or y or both are either not a number, infinite or equal to zero.
    if x.is_zero() {
        return if y.is_zero() {
            if x.is_sign_negative() && y.is_sign_negative() {
                x
            } else {
                f256::ZERO
            }
        } else {
            y
        };
    }
    if y.is_zero() {
        return x;
    }
    if x.is_nan() || y.is_nan() {
        return f256::NAN;
    }
    if x.is_infinite() {
        return if (x.bits.hi ^ y.bits.hi) == HI_SIGN_MASK {
            // x and y are infinite and have different signs
            f256::NAN
        } else {
            x
        };
    }
    // x is a number and y is infinite
    y
}

type InplaceBinOp = fn(&mut u256, &u256);
const SIGNIF_OPS: [InplaceBinOp; 2] = [u256::iadd, u256::isub];

#[inline]
pub(crate) fn add(x: f256, y: f256) -> f256 {
    // Both operands are finite and non-zero.
    // Compare the absolute values of the operands and swap them in case x < y.
    let mut a: f256;
    let mut b: f256;
    if x.abs() >= y.abs() {
        a = x;
        b = y;
    } else {
        a = y;
        b = x;
    }
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
    let sticky_bit =
        !(adj == 0 || (b_signif << (u256::BITS - adj) as usize).is_zero());
    b_signif >>= adj as usize;
    b_signif.lo |= sticky_bit as u128;
    // Determine the actual op to be performed: if the sign of the operands
    // differ, it's a subtraction, otherwise an addition.
    let signs_differ = (((x.bits.hi ^ y.bits.hi) & HI_SIGN_MASK) != 0);
    let op = SIGNIF_OPS[signs_differ as usize];
    op(&mut a_signif, &b_signif);
    // If addition carried over, right-shift the significand and increment
    // the exponent.
    if (a_signif.hi >> (HI_FRACTION_BITS + 4)) != 0 {
        a_signif >>= 1;
        a_signif.lo |= sticky_bit as u128;
        a_exp += 1;
    } else {
        // If subtraction cancelled the hidden bit or x and y are subnormal,
        // left-shift the significand and decrement the exponent. We limit the
        // adjustment to avoid the biased exponent to become negative.
        let adj = min(SIGNIFICAND_BITS + 3 - a_signif.leading_zeros(), a_exp);
        a_signif <<= adj as usize;
        a_exp -= adj;
    }
    // If the result overflows the range of values representable as `f256`,
    // return +/- Infinity.
    if a_exp >= EXP_MAX {
        return [f256::INFINITY, f256::NEG_INFINITY][a.sign() as usize];
    }
    // Determine carry from round, guard and sticky bit.
    let carry = (a_signif.lo & 0x7_u128) > 0x4;
    // Shift significand back, erase hidden bit and set exponent and sign.
    let mut bits = a_signif >> 3;
    bits.hi &= HI_FRACTION_MASK;
    bits.hi |= (a_exp as u128) << HI_FRACTION_BITS;
    bits.hi |= a.bits.hi & HI_SIGN_MASK;
    // Final rounding. Possibly overflowing into the exponent, but that is ok.
    if carry {
        bits.incr();
    }
    f256 { bits }
}

impl Add for f256 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        if self.is_special() || rhs.is_special() {
            add_special(self, rhs)
        } else {
            add(self, rhs)
        }
    }
}

forward_ref_binop!(impl Add, add);

impl Sub for f256 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        if self.is_special() || rhs.is_special() {
            add_special(self, -rhs)
        } else {
            add(self, -rhs)
        }
    }
}

forward_ref_binop!(impl Sub, sub);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_nan() {
        assert!((f256::NAN + f256::ONE).is_nan());
        assert!((f256::ONE + f256::NAN).is_nan());
        assert!((f256::NAN + f256::NAN).is_nan());
        assert!((f256::NAN + f256::INFINITY).is_nan());
        assert!((f256::INFINITY + f256::NAN).is_nan());
        assert!((f256::NAN + f256::NEG_INFINITY).is_nan());
        assert!((f256::NEG_INFINITY + f256::NAN).is_nan());
    }

    #[test]
    fn test_sub_nan() {
        assert!((f256::NAN - f256::ONE).is_nan());
        assert!((f256::ONE - f256::NAN).is_nan());
        assert!((f256::NAN - f256::NAN).is_nan());
        assert!((f256::NAN - f256::INFINITY).is_nan());
        assert!((f256::INFINITY - f256::NAN).is_nan());
        assert!((f256::NAN - f256::NEG_INFINITY).is_nan());
        assert!((f256::NEG_INFINITY - f256::NAN).is_nan());
    }

    #[test]
    fn test_add_inf() {
        assert_eq!(f256::INFINITY + f256::INFINITY, f256::INFINITY);
        assert_eq!(f256::INFINITY + f256::ONE, f256::INFINITY);
        assert_eq!(f256::ONE + f256::INFINITY, f256::INFINITY);
        assert_eq!(f256::NEG_INFINITY + f256::NEG_INFINITY, f256::NEG_INFINITY);
        assert_eq!(f256::NEG_INFINITY + f256::ONE, f256::NEG_INFINITY);
        assert_eq!(f256::ONE + f256::NEG_INFINITY, f256::NEG_INFINITY);
        assert!((f256::INFINITY + f256::NEG_INFINITY).is_nan());
        assert!((f256::NEG_INFINITY + f256::INFINITY).is_nan());
    }

    #[test]
    fn test_sub_inf() {
        assert_eq!(f256::INFINITY - f256::NEG_INFINITY, f256::INFINITY);
        assert_eq!(f256::INFINITY - f256::ONE, f256::INFINITY);
        assert_eq!(f256::ONE - f256::INFINITY, f256::NEG_INFINITY);
        assert_eq!(f256::NEG_INFINITY - f256::INFINITY, f256::NEG_INFINITY);
        assert_eq!(f256::NEG_INFINITY - f256::ONE, f256::NEG_INFINITY);
        assert_eq!(f256::ONE - f256::NEG_INFINITY, f256::INFINITY);
        assert!((f256::INFINITY - f256::INFINITY).is_nan());
        assert!((f256::NEG_INFINITY - f256::NEG_INFINITY).is_nan());
    }

    #[test]
    fn test_add_zero() {
        assert_eq!(f256::ZERO + f256::ZERO, f256::ZERO);
        assert_eq!(f256::ZERO + f256::NEG_ZERO, f256::ZERO);
        assert_eq!(f256::NEG_ZERO + f256::ZERO, f256::ZERO);
        assert_eq!(f256::NEG_ZERO + f256::NEG_ZERO, f256::NEG_ZERO);
        assert_eq!(f256::ONE + f256::ZERO, f256::ONE);
        assert_eq!(f256::ZERO + f256::ONE, f256::ONE);
        assert_eq!(f256::ONE + f256::NEG_ZERO, f256::ONE);
        assert_eq!(f256::NEG_ZERO + f256::ONE, f256::ONE);
    }

    #[test]
    fn test_pos_add_pos() {
        assert_eq!(f256::ONE + f256::ONE, f256::TWO);
        assert_eq!(f256::TWO + f256::TWO, f256::from(4.0));
    }
}
