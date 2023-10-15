// ---------------------------------------------------------------------------
// Copyright:   (c) 2023 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use std::{cmp::min, ops::Rem};

use crate::{
    abs_bits,
    big_uint::u256,
    consts::{FRAC_3_PI_2, FRAC_PI_2, PI, TAU},
    exp_bits, f256,
    math::FP248,
    norm_bit, sign_bits_hi, signif, EXP_BIAS, EXP_BITS, FRACTION_BITS,
    HI_ABS_MASK, HI_EXP_MASK,
};

// Number of bits to shift left for adjusting the radix point from f256 to
// FP248
const PREC_ADJ: u32 = FP248::FRACTION_BITS - FRACTION_BITS;
const EXP_BIAS_ADJ: u32 = EXP_BIAS - PREC_ADJ;

// Cut-off for small values
// ≈0.00000000000000000000000000000000000210094754024801845063812748106760843
const SMALL_CUT_OFF: u256 = u256::new(
    0x3ff8865752be2a167f0644b50757a602,
    0x81800000000000000000000000000000,
);

// Bounds of the quarters of the unit circle (fixed with 248 fractional bits)
const FP_HALF_PI: u256 = signif(&FRAC_PI_2.bits).shift_left(PREC_ADJ);
const FP_PI: u256 = (&signif(&PI.bits)).shift_left(PREC_ADJ + 1);
const FP_3_PI_HALF: u256 =
    (&signif(&FRAC_3_PI_2.bits)).shift_left(PREC_ADJ + 2);
const FP_TAU: u256 = (&signif(&TAU.bits)).shift_left(PREC_ADJ + 2);

fn div_rem_half_pi(abs_bits_x: &u256) -> (u32, FP248) {
    let exp_x = exp_bits(&abs_bits_x) as i32 - EXP_BIAS as i32;
    let sh = exp_x + PREC_ADJ as i32;
    let (quadrant, mut x_rem_half_pi) = match sh {
        ..=0 => {
            // -236 <= e <= -12
            (0_u32, &signif(&abs_bits_x) >> (-sh) as u32)
        }
        1..=11 => {
            // -11 <= e < 0
            (0_u32, &signif(&abs_bits_x) << sh as u32)
        }
        12 => {
            // e = 0
            let mut fp_x_signif = &signif(&abs_bits_x) << PREC_ADJ;
            if fp_x_signif >= FP_HALF_PI {
                (1_u32, &fp_x_signif - &FP_HALF_PI)
            } else {
                (0_u32, fp_x_signif)
            }
        }
        _ => {
            // 0 < e <= EMAX
            let x_rem_tau =
                signif(&abs_bits_x).lshift_rem(&FP_TAU, sh as u32);
            if &x_rem_tau < &FP_HALF_PI {
                (0, x_rem_tau)
            } else if &x_rem_tau < &FP_PI {
                (1, &x_rem_tau - &FP_HALF_PI)
            } else if &x_rem_tau < &FP_3_PI_HALF {
                (2, &x_rem_tau - &FP_PI)
            } else {
                (3, &x_rem_tau - &FP_3_PI_HALF)
            }
        }
    };
    let fp_x_rem_half_pi = FP248 {
        sign: 0,
        signif: x_rem_half_pi,
    };
    (quadrant, fp_x_rem_half_pi)
}

impl f256 {
    /// Simultaneously computes the sine and cosine of the number x.
    ///
    /// Returns (sin(x), cos(x)).
    pub fn sin_cos(&self) -> (Self, Self) {
        let abs_bits_x = abs_bits(&self);
        // If x is NAN or infinite, both, sine x and cosine x, are NAN.
        if abs_bits_x.hi >= HI_EXP_MASK {
            return (f256::NAN, f256::NAN);
        }
        // If |x| is very small, sine x == x and cosine x == 1.
        if abs_bits_x <= SMALL_CUT_OFF {
            return (*self, f256::ONE);
        }
        // Now we have ε < |x| < ∞.
        // x = (-1)ˢ × m × 2ᵉ with 1 <= m < 2 and e >= -236
        // Calculate (|x| / ½π) % 4 and |x| % ½π, adjusted to 248 fractional
        // digits.
        let (quadrant, fp_x_rem_half_pi) = div_rem_half_pi(&abs_bits_x);
        // Calc (sin, cos) of |x| % ½π.
        let (fp_sin_x, fp_cos_x) = fp_x_rem_half_pi.sin_cos(quadrant);
        let mut sin_x = f256::from(&fp_sin_x);
        let cos_x = f256::from(&fp_cos_x);
        // sin(-x) = -sin(x)
        sin_x.bits.hi ^= sign_bits_hi(&self);
        (sin_x, cos_x)
    }

    /// Computes the sine of a number (in radians).
    #[inline(always)]
    pub fn sin(&self) -> Self {
        self.sin_cos().0
    }

    /// Computes the cosine of a number (in radians).
    #[inline(always)]
    pub fn cos(&self) -> Self {
        self.sin_cos().1
    }
}

#[cfg(test)]
mod sin_cos_tests {
    use core::str::FromStr;

    use super::*;
    use crate::{
        consts::{FRAC_PI_3, FRAC_PI_4, FRAC_PI_6},
        ONE_HALF,
    };

    #[test]
    fn test_frac_pi_2_multiples() {
        const EXPECTED: [(f256, f256); 4] = [
            (f256::ZERO, f256::ONE),
            (f256::ONE, f256::ZERO),
            (f256::ZERO, f256::NEG_ONE),
            (f256::NEG_ONE, f256::ZERO),
        ];
        for i in 0_u32..=4_u32 {
            let f = f256::from(i) * FRAC_PI_2;
            let (sin, cos) = f.sin_cos();
            assert_eq!((sin, cos), EXPECTED[(i % 4) as usize]);
        }
    }

    #[test]
    fn test_signs() {
        let p = FRAC_PI_4;
        for i in 0..9 {
            let f = f256::from(i) * FRAC_PI_2 + p;
            let (sin, cos) = f.sin_cos();
            let quadrant = (i % 4) + 1;
            match quadrant {
                1 => {
                    assert!(sin.is_sign_positive());
                    assert!(cos.is_sign_positive());
                }
                2 => {
                    assert!(sin.is_sign_positive());
                    assert!(cos.is_sign_negative());
                }
                3 => {
                    assert!(sin.is_sign_negative());
                    assert!(cos.is_sign_negative());
                }
                4 => {
                    assert!(sin.is_sign_negative());
                    assert!(cos.is_sign_positive());
                }
                _ => unreachable!(),
            }
        }
    }

    #[test]
    fn test_frac_pi_4() {
        // sin(45°) = cos(45°)
        let f = FRAC_PI_4;
        let (sin, cos) = f.sin_cos();
        let d = cos - sin;
        assert!(d < f256::EPSILON);
    }

    #[test]
    fn test_frac_pi_3_and_frac_pi_6() {
        // sin(30°) = 0.5
        let sin = FRAC_PI_6.sin();
        assert_eq!(sin, ONE_HALF);
        // cos(60°) = 0.5
        let cos = FRAC_PI_3.cos();
        assert_eq!(cos, ONE_HALF);
        // sin(60°) = cos(30°)
        let sin = FRAC_PI_3.sin();
        let cos = FRAC_PI_6.cos();
        let d = sin - cos;
        assert!(d < f256::EPSILON);
    }

    #[test]
    fn test_neg_values() {
        for f in [f256::MIN, -FRAC_PI_3, f256::NEG_ONE] {
            assert_eq!(f.sin(), -f.abs().sin());
            assert_eq!(f.cos(), f.abs().cos());
        }
    }

    #[test]
    fn test_continuity_near_zero() {
        let c = f256 {
            bits: SMALL_CUT_OFF,
        };
        let d = f256::encode(0, c.exponent(), u256::new(0, 1));
        let mut f = c;
        let mut g = f;
        for i in 0..1000 {
            g += d;
            assert!(f < g);
            assert!(f.sin() <= g.sin());
            assert!(f.cos() >= g.cos());
            f = g;
        }
    }

    #[test]
    fn test_continuity_near_one() {
        let c = f256::ONE;
        let d = f256::EPSILON;
        let mut f = c;
        let mut g = f;
        for i in 0..10 {
            g += d;
            assert!(f < g);
            assert!(f.sin() <= g.sin());
            assert!(f.cos() >= g.cos());
            f = g;
        }
    }

    #[test]
    fn test_continuity_near_three() {
        let c = f256::from(3);
        let d = f256::EPSILON * f256::TWO;
        let mut f = c;
        let mut g = f;
        for i in 0..10 {
            g += d;
            assert!(f < g);
            assert!(f.sin() >= g.sin());
            assert!(f.cos() <= g.cos());
            f = g;
        }
    }
}
