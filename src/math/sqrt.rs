// ---------------------------------------------------------------------------
// Copyright:   (c) 2023 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use crate::{
    f256, u256, EMIN, EXP_BIAS, EXP_BITS, EXP_MAX, HI_FRACTION_BIAS,
    HI_FRACTION_BITS, SIGNIFICAND_BITS,
};

impl f256 {
    /// Returns the square root of a `self`.
    ///
    /// Returns NaN if `self` is a negative number other than `-0.0`.
    ///
    /// # Examples
    ///
    /// ```
    // let positive = 4.0_f64;
    // let negative = -4.0_f64;
    // let negative_zero = -0.0_f64;
    //
    // let abs_difference = (positive.sqrt() - 2.0).abs();
    //
    // assert!(abs_difference < 1e-10);
    // assert!(negative.sqrt().is_nan());
    // assert!(negative_zero.sqrt() == negative_zero);
    /// ```
    pub fn sqrt(self) -> Self {
        // Check whether `self` is negative or ∈ {-0, +0, +∞, NAN}.
        if self.bits > f256::NEG_ZERO.bits {
            // `self` < 0
            return f256::NAN;
        }
        if self.is_special() {
            // `self` either not a number, infinite or equal to zero.
            return self;
        }

        // `self` is (sub-)normal and positive
        let biased_exp = self.bits.hi >> HI_FRACTION_BITS;
        let hidden_bit = (biased_exp != 0) as i32;
        let norm_shift = self.bits.leading_zeros().saturating_sub(EXP_BITS);
        // Calculate the exponent
        let mut exp = biased_exp as i32 + EMIN - hidden_bit - norm_shift as i32;
        let odd_exp = exp & 1;
        exp = (exp - odd_exp) / 2;
        // Calculate the significand, gain extra bit for final rounding
        let mut signif = &self.fraction() << norm_shift;
        signif.hi |= (hidden_bit as u128) << HI_FRACTION_BITS;
        let mut q = u256::new(HI_FRACTION_BIAS << 1, 0);
        let mut r = &(&signif << (1 + odd_exp as u32)) - &q;
        let mut s = q;
        for i in 1..SIGNIFICAND_BITS {
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
        Self::new(q, (exp + EXP_BIAS as i32) as u32, 0)
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
    fn test_subnormal() {
        let f = f256 {
            bits: u256 {
                hi: 161381583805889998189973969922,
                lo: 288413346707470246106660640932215474040,
            },
        };
        assert!(f.is_subnormal());
        let r = f256 {
            bits: u256 {
                hi: 42533487390635923064310396803489994282,
                lo: 251643572745990121674876797336685460939,
            },
        };
        assert!(r.is_normal());
        assert_eq!(f.sqrt(), r);
        assert_eq!(r * r, f);
    }
}
