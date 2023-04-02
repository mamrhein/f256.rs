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

// TODO: use core::cmp::min when it got stable in const context
const fn min(x: u32, y: u32) -> u32 {
    x - x.saturating_sub(y)
}

trait Float: Copy + Clone {
    /// Precision level in relation to single precision float (f32)
    const PREC_LEVEL: u32;
    /// Total number of bits
    const TOTAL_BITS: u32 = 1_u32 << Self::PREC_LEVEL;
    /// Number of exponent bits
    const EXP_BITS: u32 = 4 * Self::PREC_LEVEL - min(6, Self::PREC_LEVEL) - 7;
    /// Number of fraction bits
    const FRACTION_BITS: u32 = Self::TOTAL_BITS - Self::EXP_BITS - 1;
    /// Maximum value of biased base 2 exponent
    const BIASED_EXP_MAX: u32 = (1_u32 << Self::EXP_BITS) - 1;
    /// Base 2 exponent bias (incl. radix adjustment)
    const EXP_BIAS_ADJ: u32 =
        (Self::BIASED_EXP_MAX >> 1) + Self::FRACTION_BITS;
    /// Fraction mask
    const FRACTION_MASK: u64 = (1_u64 << Self::FRACTION_BITS) - 1;
    /// Fraction bias
    const FRACTION_BIAS: u64 = 1_u64 << Self::FRACTION_BITS;
    /// Number of bits to shift right for sign
    const SIGN_SHIFT: u32 = Self::TOTAL_BITS - 1;
    /// Sign mask
    const SIGN_MASK: u64 = 1_u64 << Self::SIGN_SHIFT;
    /// Abs mask
    const ABS_MASK: u64 = !Self::SIGN_MASK;
    /// Bit representation of +Inf
    const INF: u64 = (Self::BIASED_EXP_MAX as u64) << Self::FRACTION_BITS;
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

        if abs_bits >= F::FRACTION_BIAS && abs_bits < F::INF {
            // Normal value
            let exp = (abs_bits >> F::FRACTION_BITS as u64) as i32
                - F::EXP_BIAS_ADJ as i32;
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
                1 - F::EXP_BIAS_ADJ as i32,
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
        let x = f256::from(3.5_f64);
        assert_eq!(x.as_sign_exp_signif(), (0, -1, (0, 7)));
        let x = f256::from(-17.625_f64);
        assert_eq!(x.as_sign_exp_signif(), (1, -3, (0, 141)));
        let x = f256::from(0.0625_f64);
        assert_eq!(x.as_sign_exp_signif(), (0, -4, (0, 1)));
        let x = f256::from(109.04e-115_f64);
        assert_eq!(x.as_sign_exp_signif(), (0, -428, (0, 7558297586173341)));
    }

    #[test]
    fn test_subnormal_values() {
        let x = f256::from(7.4e-317_f64);
        assert_eq!(x.as_sign_exp_signif(), (0, -1074, (0, 14977767)));
        let x = f256::from(-0.984e-312_f64);
        assert_eq!(x.as_sign_exp_signif(), (1, -1073, (0, 99581908627)));
    }
}

#[cfg(test)]
mod from_f32_tests {
    use super::*;

    #[test]
    fn test_nan() {
        assert!(f256::from(f32::NAN).is_nan());
        assert!(f256::from(-f32::NAN).is_nan());
    }

    #[test]
    fn test_inf() {
        assert_eq!(f256::from(f32::INFINITY), f256::INFINITY);
        assert_eq!(f256::from(f32::NEG_INFINITY), f256::NEG_INFINITY);
    }

    #[test]
    fn test_zero() {
        assert_eq!(f256::from(0_f32), f256::ZERO);
        assert_eq!(f256::from(-0_f32), f256::NEG_ZERO);
    }

    #[test]
    fn test_normal_values() {
        assert_eq!(f256::from(1_f32), f256::ONE);
        assert_eq!(f256::from(-1_f32), f256::NEG_ONE);
        assert_eq!(f256::from(2_f32), f256::TWO);
        let x = f256::from(3.5_f32);
        assert_eq!(x.as_sign_exp_signif(), (0, -1, (0, 7)));
        let x = f256::from(-17.625_f32);
        assert_eq!(x.as_sign_exp_signif(), (1, -3, (0, 141)));
        let x = f256::from(0.0625_f32);
        assert_eq!(x.as_sign_exp_signif(), (0, -4, (0, 1)));
        let x = f256::from(3.782e-38_f32);
        assert_eq!(x.as_sign_exp_signif(), (0, -148, (0, 13494627)));
    }

    #[test]
    fn test_subnormal_values() {
        let x = f256::from(7.4e-317);
        assert_eq!(x.as_sign_exp_signif(), (0, -1074, (0, 14977767)));
        let x = f256::from(-0.984e-312);
        assert_eq!(x.as_sign_exp_signif(), (1, -1073, (0, 99581908627)));
    }
}
