// ---------------------------------------------------------------------------
// Copyright:   (c) 2022 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use core::ops::Neg;

use crate::{f256, u256};

trait Float: Copy + Clone {
    /// Precision level in relation to single precision float (f32)
    const PREC_LEVEL: u32;
    /// Total number of bits
    const TOTAL_BITS: u32 = 1_u32 << Self::PREC_LEVEL;
    /// Number of exponent bits
    const EXP_BITS: u32 = 4 * Self::PREC_LEVEL - 13;
    /// Number of significand bits
    const SIGNIFICAND_BITS: u32 = Self::TOTAL_BITS - Self::EXP_BITS;
    /// Number of fraction bits
    const FRACTION_BITS: u32 = Self::SIGNIFICAND_BITS - 1;
    /// Maximum value of biased base 2 exponent
    const EXP_MAX: u32 = (1_u32 << Self::EXP_BITS) - 1;
    /// Base 2 exponent bias (incl. radix adjustment)
    const EXP_BIAS: u32 = (Self::EXP_MAX >> 1) + Self::FRACTION_BITS;
    /// Fraction mask
    const FRACTION_MASK: u64 = (1_u64 << Self::FRACTION_BITS) - 1;
    /// Fraction bias
    const FRACTION_BIAS: u64 = 1_u64 << Self::FRACTION_BITS;
    /// Number of bits to shift right for sign
    const SIGN_SHIFT: u32 = Self::TOTAL_BITS - 1;
    // Sign mask
    const SIGN_MASK: u64 = 1_u64 << Self::SIGN_SHIFT;
    /// Abs mask
    const ABS_MASK: u64 = !Self::SIGN_MASK;
    // Bit representation of +Inf
    const INF: u64 = (Self::EXP_MAX as u64) << Self::FRACTION_BITS;
    // Bit representation of maximum normal value
    const MAX: u64 = (((Self::EXP_MAX - 1) as u64) << Self::FRACTION_BITS)
        | Self::FRACTION_MASK;
    /// Raw transmutation to u64.
    fn to_bits(self) -> u64;
}

impl Float for f32 {
    const PREC_LEVEL: u32 = 5;

    #[inline]
    fn to_bits(self) -> u64 {
        self.to_bits() as u64
    }
}

impl Float for f64 {
    const PREC_LEVEL: u32 = 6;

    #[inline]
    fn to_bits(self) -> u64 {
        self.to_bits()
    }
}

impl<F: Float> From<F> for f256 {
    #[allow(clippy::cast_possible_wrap)]
    #[allow(clippy::cast_possible_truncation)]
    fn from(f: F) -> Self {
        let bits = f.to_bits();
        let abs_bits = bits & F::ABS_MASK;
        let sign = (bits >> F::SIGN_SHIFT) as u32;

        if abs_bits.wrapping_sub(1) < F::MAX {
            // Normal value
            let exp = ((bits >> F::FRACTION_BITS as u64) & F::EXP_MAX as u64)
                as u32 as i32
                - F::EXP_BIAS as i32;
            let significand = u256 {
                hi: 0,
                lo: ((bits & F::FRACTION_MASK) | F::FRACTION_BIAS) as u128,
            };
            Self::encode(sign, exp, significand)
        } else if abs_bits == 0 {
            // +/- zero
            [Self::ZERO, Self::NEG_ZERO][sign as usize]
        } else if abs_bits < F::FRACTION_BIAS {
            // subnormal
            Self::encode(
                sign,
                1 - F::EXP_BIAS as i32,
                u256 {
                    hi: 0,
                    lo: (bits & F::FRACTION_MASK) as u128,
                },
            )
        } else if abs_bits == F::INF {
            // +/- inf
            [Self::INFINITY, Self::NEG_INFINITY][sign as usize]
        } else {
            // +/- NaN
            [Self::NAN, -Self::NAN][sign as usize]
        }
    }
}

#[cfg(test)]
mod from_f64_tests {
    use super::*;

    #[test]
    fn test_nan() {
        assert!(f256::from(f64::NAN).is_nan());
        assert!(f256::from(-f64::NAN).is_nan());
    }

    #[test]
    fn test_inf() {
        assert_eq!(f256::from(f64::INFINITY), f256::INFINITY);
        assert_eq!(f256::from(f64::NEG_INFINITY), f256::NEG_INFINITY);
    }

    #[test]
    fn test_zero() {
        assert_eq!(f256::from(0_f64), f256::ZERO);
        assert_eq!(f256::from(-0_f64), f256::NEG_ZERO);
    }

    #[test]
    fn test_normal_values() {
        assert_eq!(f256::from(1_f64), f256::ONE);
        assert_eq!(f256::from(-1_f64), f256::NEG_ONE);
        assert_eq!(f256::from(2_f64), f256::TWO);
        let x = f256::from(3.5);
        assert_eq!(x.decode(), (0, -1, u256::new(0, 7)));
        let x = f256::from(-17.625);
        assert_eq!(x.decode(), (1, -3, u256::new(0, 141)));
    }

    // TODO: test subnormal values
}
