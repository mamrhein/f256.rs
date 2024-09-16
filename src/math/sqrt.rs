// ---------------------------------------------------------------------------
// Copyright:   (c) 2023 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use crate::{
    abs_bits, exp_bits, f256, fraction, BinEncSpecial, EMIN, EXP_BIAS,
    EXP_BITS, EXP_MAX, HI_FRACTION_BIAS, HI_FRACTION_BITS, SIGNIFICAND_BITS,
    U256,
};

impl f256 {
    /// Returns the square root of `self`.
    ///
    /// Returns NaN if `self` is a negative number other than `-0.0`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use f256::f256;
    /// let f = f256::from(1822500);
    /// assert_eq!(f.sqrt(), f256::from(1350));
    /// assert!(f256::NEG_ONE.sqrt().is_nan());
    /// assert_eq!(f256::NEG_ZERO.sqrt(), f256::NEG_ZERO);
    /// ```
    #[must_use]
    #[allow(clippy::integer_division)]
    #[allow(clippy::cast_possible_wrap)]
    #[allow(clippy::cast_sign_loss)]
    pub fn sqrt(self) -> Self {
        let bin_enc = self.bits;
        // Check whether `self` is negative or ∈ {-0, +0, +∞, NAN}.
        if bin_enc > Self::NEG_ZERO.bits {
            // `self` < 0
            return Self::NAN;
        }
        if bin_enc.is_special() {
            // `self` either not a number, infinite or equal to zero.
            return self;
        }

        // `self` is (sub-)normal and positive
        let biased_exp = exp_bits(&bin_enc);
        let hidden_bit = (biased_exp != 0) as i32;
        let norm_shift = bin_enc.leading_zeros().saturating_sub(EXP_BITS);
        // Calculate the exponent
        let mut exp =
            biased_exp as i32 + EMIN - hidden_bit - norm_shift as i32;
        let exp_is_odd = exp & 1;
        exp = (exp - exp_is_odd) / 2;
        // Calculate the significand, gain extra bit for final rounding
        let mut signif = &fraction(&bin_enc) << norm_shift;
        signif.hi |= (hidden_bit as u128) << HI_FRACTION_BITS;
        let mut q = U256::new(HI_FRACTION_BIAS << 1, 0);
        let mut r = &(&signif << (1 + exp_is_odd as u32)) - &q;
        let mut s = q;
        for i in 1..=SIGNIFICAND_BITS {
            s >>= 1;
            let t = &r << 1;
            let u = &(&q << 1) + &s;
            if t < u {
                r = t;
            } else {
                q += &s;
                r = &t - &u;
            }
        }
        // Final rounding
        q = &(&q >> 1) + ((q.lo & 1) as u32);
        Self::new(0, exp, q)
    }
}

#[cfg(test)]
mod sqrt_tests {
    use core::str::FromStr;

    use super::*;
    use crate::consts::{PI, SQRT_2, SQRT_5, SQRT_PI};

    #[test]
    fn test_zero() {
        assert_eq!(f256::ZERO.sqrt(), f256::ZERO);
        assert_eq!(f256::NEG_ZERO.sqrt(), f256::NEG_ZERO);
    }

    #[test]
    fn test_inf() {
        assert_eq!(f256::INFINITY.sqrt(), f256::INFINITY);
        assert!(f256::NEG_INFINITY.sqrt().is_nan());
    }

    #[test]
    fn test_nan() {
        assert!(f256::NAN.sqrt().is_nan());
    }

    #[test]
    fn test_neg_values() {
        assert!(f256::NEG_ONE.sqrt().is_nan());
        assert!(f256::TEN.negated().sqrt().is_nan());
        assert!(f256::MIN_GT_ZERO.negated().sqrt().is_nan());
        assert!(f256::from(-290317).sqrt().is_nan());
    }

    #[test]
    fn test_exact_squares() {
        let f = f256::from(81);
        assert_eq!(f.sqrt(), f256::from(9));
        let f = f256::from_str("157836662403.890625").unwrap();
        assert_eq!(f.sqrt(), f256::from_str("397286.625").unwrap());
    }

    #[test]
    fn test_two() {
        let sqrt2 = f256::TWO.sqrt();
        assert_eq!(sqrt2, SQRT_2);
    }

    #[test]
    fn test_five() {
        let sqrt5 = f256::from(5).sqrt();
        assert_eq!(sqrt5, SQRT_5);
    }

    #[test]
    fn test_pi() {
        let sqrt_pi = PI.sqrt();
        assert_eq!(sqrt_pi, SQRT_PI);
    }

    #[test]
    fn test_normal_1() {
        let f = f256::from(7_f64);
        let r = f256::from_sign_exp_signif(
            0,
            -235,
            (
                429297694403283601796750956887579,
                277843259545175179498338411277842904177,
            ),
        );
        assert_ne!(f, r * r);
        assert_eq!(f.sqrt(), r);
    }

    #[test]
    fn test_normal_2() {
        let f = f256::from_sign_exp_signif(
            0,
            -262021,
            (0, 73913349228891354865085158512847),
        );
        assert!(f.is_normal());
        let r = f256::from_sign_exp_signif(
            0,
            -131194,
            (
                438052537377059491661973478527305,
                282106124646787902904225457342964901703,
            ),
        );
        assert!(r.is_normal());
        assert_eq!(r * r, f);
        assert_eq!(f.sqrt(), r);
    }

    #[test]
    fn test_normal_3() {
        let f = f256::from_sign_exp_signif(
            0,
            157426,
            (
                6224727460272857694717553232696,
                192855907509048186086344977196907424065,
            ),
        );
        assert!(f.is_normal());
        let r = f256::from_sign_exp_signif(
            0,
            78594,
            (
                89889700240364350456294468037203,
                107344220596675717575864825763718692041,
            ),
        );
        assert!(r.is_normal());
        assert_eq!(r * r, f);
        assert_eq!(f.sqrt(), r);
    }

    #[test]
    fn test_subnormal_1() {
        let f = f256 {
            bits: U256 {
                hi: 161381583805889998189973969922,
                lo: 288413346707470246106660640932215474040,
            },
        };
        assert!(f.is_subnormal());
        let r = f256 {
            bits: U256 {
                hi: 42533487390635923064310396803489994282,
                lo: 251643572745990121674876797336685460940,
            },
        };
        assert!(r.is_normal());
        assert_eq!(r * r, f);
        assert_eq!(f.sqrt(), r);
    }
}
