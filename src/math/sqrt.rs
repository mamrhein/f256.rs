// ---------------------------------------------------------------------------
// Copyright:   (c) 2023 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use crate::{
    abs_bits, exp_bits, f256, fraction, BigUInt, BinEncSpecial, HiLo, EMIN,
    EXP_BIAS, EXP_BITS, EXP_MAX, HI_FRACTION_BIAS, HI_FRACTION_BITS,
    SIGNIFICAND_BITS, U256, U512,
};
use core::ops::{Add, Shr};

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
        let mut signif = fraction(&bin_enc) << norm_shift;
        signif.hi.0 |= (hidden_bit as u128) << HI_FRACTION_BITS;
        // N: number of fractional digits
        // e: base 2 exponent of the input value
        // p: base 2 exponent of the root
        // a: adjustment for the significand
        let mut e = biased_exp as i32 + EMIN - hidden_bit - norm_shift as i32;
        let a = e & 1;
        // The following subtraction is neccessary to get the correct
        // quotient with a positive remainder for negative exponents!
        let p = (e - a) / 2;
        let m = signif << (1 + a as u32);
        // Now we have
        // self = m⋅2⁻¹⋅2⁻ᴺ⋅2ᵉ⁻ᵅ
        // and
        // √self = √(m⋅2⁻¹⋅2⁻ᴺ)⋅2ᵖ
        // Let M = m⋅2⁻¹⋅2⁻ᴺ
        // The following algorithm calculates the significand of the resulting
        // root bit by bit, one per iteration, starting with 1.
        // It is described in detail in
        // J.-M. Muller et al., Handbook of Floating-Point Arithmetic, 2. ed.,
        // Chapter 9.5.3.1
        // Qᵢ: root extracted thus far
        // Rᵢ: remainder
        // Invariant: M = Qᵢ² + Rᵢ
        // Q₀ = 1
        // q₀ = Q₀⋅2ᴺ⁺¹
        let mut q = U256::new(HI_FRACTION_BIAS << 1, 0);
        // R₀ = M - Q₀²
        // r₀ = R₀⋅2ᴺ⁺¹ = (m⋅2⁻¹⋅2⁻ᴺ - (q₀⋅2⁻¹⋅2⁻ᴺ)²)⋅2ᴺ⁺¹ = m - q₀²⋅2⁻¹⋅2⁻ᴺ
        // Q₀ = 1 => q₀²⋅2⁻¹⋅2⁻ᴺ = q₀ => r₀ = m - q₀
        let mut r = m - q;
        // Qᵢ = qᵢ⋅2⁻¹⋅2⁻ᴺ
        // Rᵢ = rᵢ⋅2⁻¹⋅2⁻ᴺ⋅2⁻ⁱ
        // M = Qᵢ² + Rᵢ
        // => m⋅2⁻¹⋅2⁻ᴺ = (qᵢ⋅2⁻¹⋅2⁻ᴺ)² + rᵢ⋅2⁻¹⋅2⁻ᴺ⋅2⁻ⁱ
        // => m⋅2⁻¹⋅2⁻ᴺ = (qᵢ²⋅2⁻¹⋅2⁻ᴺ + rᵢ⋅2⁻ⁱ)⋅2⁻¹⋅2⁻ᴺ
        // => m = qᵢ²⋅2⁻¹⋅2⁻ᴺ + rᵢ⋅2⁻ⁱ
        if cfg!(debug_assertions) {
            let q2 = q.widening_mul(&q);
            debug_assert_eq!(
                m,
                U512::from_hi_lo(q2.1, q2.0)
                    .shr(SIGNIFICAND_BITS)
                    .lo_t()
                    .add(r)
            );
        };
        let mut s = q;
        for i in 1..=SIGNIFICAND_BITS {
            if r.is_zero() {
                break;
            }
            // Next bit
            // Sᵢ = 2⁻ⁱ
            // sᵢ = Sᵢ⋅2ᴺ⁺¹ = 2ᴺ⁺¹⁻ⁱ
            s >>= 1;
            // Tentative next estimation
            // Qᵢ = Qᵢ₋₁ + Sᵢ
            // qᵢ = Qᵢ⋅2ᴺ⁺¹ = Qᵢ₋₁⋅2ᴺ⁺¹ + Sᵢ⋅2ᴺ⁺¹ = qᵢ₋₁ + sᵢ
            // Tentative remainder
            // Tᵢ = M - Qᵢ²
            // tᵢ = Tᵢ⋅2ᴺ⁺¹⁺ⁱ
            //    = (M - Qᵢ²)⋅2ᴺ⁺¹⁺ⁱ
            //    = (M - (Qᵢ₋₁ + Sᵢ)²)⋅2ᴺ⁺¹⁺ⁱ
            //    = (M - (Qᵢ₋₁² + 2⋅Qᵢ₋₁⋅Sᵢ + Sᵢ²))⋅2ᴺ⁺¹⁺ⁱ
            //    = 2⋅(M - Qᵢ₋₁²)⋅2ᴺ⁺¹⁺ⁱ⁻¹ - (2⋅Qᵢ₋₁⋅Sᵢ + Sᵢ²)⋅2ᴺ⁺¹⁺ⁱ
            //    = 2⋅(M - Qᵢ₋₁²)⋅2ᴺ⁺¹⁺ⁱ⁻¹ - (2⋅Qᵢ₋₁⋅2ᴺ⁺¹ + Sᵢ⋅2ᴺ⁺¹)⋅Sᵢ⋅2ⁱ
            //    = 2⋅rᵢ₋₁ - (2⋅qᵢ₋₁ + sᵢ)
            r <<= 1;
            let u = (&q << 1) + s;
            // If tᵢ >= 0 the next bit of the result is 1, else 0.
            if r >= u {
                q += &s;
                r -= &u;
                // m = yᵢ²⋅2⁻¹⋅2⁻ᴺ + rᵢ⋅2⁻ⁱ
                if cfg!(debug_assertions) {
                    let q2 = q.widening_mul(&q);
                    debug_assert!(
                        m - U512::from_hi_lo(q2.1, q2.0)
                            .shr(SIGNIFICAND_BITS)
                            .lo_t()
                            .add(r.shr(i))
                            <= U256::ONE
                    );
                };
            }
        }
        // Final reconstruction and rounding.
        // The sqare root of a floating point number can't be an exact
        // midpoint between two consecutive floating point numbers, so there
        // is no need to care about ties.
        q = q + (q.lo.0 & 1_u128);
        Self::new(0, p, q >> 1)
    }
}

#[cfg(test)]
mod sqrt_tests {
    use core::str::FromStr;

    use super::*;
    use crate::{
        consts::{FRAC_1_SQRT_2, PI, SQRT_2, SQRT_5, SQRT_PI},
        ONE_HALF,
    };

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
    fn test_one_half() {
        let r = ONE_HALF.sqrt();
        assert_eq!(r, FRAC_1_SQRT_2);
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
    fn test_nine() {
        let nine = f256::from(9);
        let three = f256::from(3);
        assert_eq!(nine.sqrt(), three);
    }

    #[test]
    fn test_nine_quarter() {
        let nine = f256::from(9);
        let four = f256::from(4);
        let three = f256::from(3);
        assert_eq!((nine / four).sqrt(), three / f256::TWO);
    }

    #[test]
    fn test_near_four() {
        let four = f256::TWO.square();
        let four_plus_ulp = four - four.ulp();
        assert_eq!(four_plus_ulp.sqrt(), f256::TWO - f256::TWO.ulp().div2());
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
            bits: U256::new(
                161381583805889998189973969922,
                288413346707470246106660640932215474040,
            ),
        };
        assert!(f.is_subnormal());
        let r = f256 {
            bits: U256::new(
                42533487390635923064310396803489994282,
                251643572745990121674876797336685460940,
            ),
        };
        assert!(r.is_normal());
        assert_eq!(r * r, f);
        assert_eq!(f.sqrt(), r);
    }
}
