// ---------------------------------------------------------------------------
// Copyright:   (c) 2023 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use core::{
    cmp::{max, min},
    ops::{Div, Rem, Shl, Shr},
};

use super::FP248;
use crate::{
    abs_bits, abs_bits_sticky,
    big_uint::u256,
    consts::{FRAC_3_PI_2, FRAC_PI_2, FRAC_PI_4, PI, TAU},
    exp_bits, f256, norm_bit, sign_bits_hi, signif, BinEncAnySpecial,
    EXP_BIAS, EXP_BITS, FRACTION_BITS, HI_ABS_MASK, HI_EXP_MASK,
    HI_FRACTION_BITS,
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

// Cut-off of exponent f{r large values
const LARGE_EXP_CUT_OFF: u32 = 240;
// Cut-off for large values (2²⁴⁰)
const LARGE_CUT_OFF: u256 = u256::new(
    ((EXP_BIAS + LARGE_EXP_CUT_OFF) as u128) << HI_FRACTION_BITS,
    0_u128,
);

// Bounds of the quarters of the unit circle (fixed with 248 fractional bits)
const FP_HALF_PI: u256 = signif(&FRAC_PI_2.bits).shift_left(PREC_ADJ);
const FP_PI: u256 = signif(&PI.bits).shift_left(PREC_ADJ + 1);
const FP_3_PI_HALF: u256 = signif(&FRAC_3_PI_2.bits).shift_left(PREC_ADJ + 2);
const FP_TAU: u256 = signif(&TAU.bits).shift_left(PREC_ADJ + 2);

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

    /// Computes the arctangent of a number (in radians).
    ///
    /// Return value is in radians in the range [-½π, ½π].
    pub fn atan(&self) -> Self {
        let abs_bits_self = abs_bits(self);
        // If self is NAN, atan self is NAN.
        if (abs_bits_self.hi | (abs_bits_self.lo != 0) as u128) > HI_EXP_MASK
        {
            return f256::NAN;
        }
        // If |self| >= 2²⁴⁰, atan self = ±½π.
        if abs_bits_self.hi >= LARGE_CUT_OFF.hi {
            let mut res = FRAC_PI_2;
            res.bits.hi ^= sign_bits_hi(self);
            return res;
        }
        // If |self| is very small, atan self = self.
        if abs_bits_self <= SMALL_CUT_OFF {
            return *self;
        }
        // Now we have ε < |self| < 2²⁴⁰.
        // self = (-1)ˢ × m × 2ᵉ with 1 <= m < 2 and -236 <= e < 240
        // Convert self into a fraction of two FP248 values, so that
        // self = y / x.
        let exp_bits_self = exp_bits(&abs_bits_self);
        let fp_signif_self = &signif(&abs_bits_self) << PREC_ADJ;
        let x = FP248 {
            sign: 0,
            signif: &FP248::ONE.signif
                >> exp_bits_self.saturating_sub(EXP_BIAS),
        };
        let y = FP248 {
            sign: self.sign(),
            signif: &fp_signif_self >> EXP_BIAS.saturating_sub(exp_bits_self),
        };
        let fp_atan_self = y.atan2(&x);
        f256::from(&fp_atan_self)
    }

    #[inline]
    fn map_atan2_signs(&self, sign_y: u32, sign_x: u32) -> Self {
        match (sign_y, sign_x) {
            (0, 0) => *self,
            (0, 1) => &PI - self,
            (1, 0) => -*self,
            _ => self - &PI,
        }
    }

    /// Computes the four quadrant arctangent of `self` (`y`) and `other`
    /// (`x`) in radians.
    ///
    /// * `x = 0`, `y = 0`: `0`
    /// * `x >= 0`: `arctan(y/x)` -> `[-½π, ½π]`
    /// * `y >= 0`: `arctan(y/x) + π` -> `(½π, π]`
    /// * `y < 0`: `arctan(y/x) - π` -> `(-π, -½π)`
    pub fn atan2(&self, other: &Self) -> Self {
        let mut abs_bits_x = abs_bits(&other);
        let mut abs_bits_y = abs_bits(&self);
        // Check whether one or both operands are NaN, infinite or zero.
        // We mask off the sign bit and mark subnormals having a significand
        // less than 2¹²⁸ in least bit of the representations high
        // u128. This allows to use only that part for the handling of
        // special cases.
        let abs_bits_sticky_x = abs_bits_sticky(&abs_bits_x);
        let abs_bits_sticky_y = abs_bits_sticky(&abs_bits_y);
        if (abs_bits_sticky_x, abs_bits_sticky_y).any_special() {
            if max(abs_bits_sticky_x, abs_bits_sticky_y) > HI_EXP_MASK {
                // Atleast one operand is NAN.
                return f256::NAN;
            }
            if abs_bits_sticky_x == 0_u128 {
                return if abs_bits_sticky_y == 0 {
                    // Both operands are zero.
                    f256::ZERO
                } else {
                    // other = 0, self != 0 => ±½π
                    let mut res = FRAC_PI_2;
                    res.bits.hi |= sign_bits_hi(&self);
                    res
                };
            }
            if abs_bits_sticky_y == 0_u128 {
                // self = 0, other != 0 => ±π
                let mut res = PI;
                res.bits.hi |= sign_bits_hi(&other);
                return res;
            }
            // Both operands are infinite.
            return FRAC_PI_4.map_atan2_signs(self.sign(), other.sign());
        }

        // Both operands are finite and non-zero.

        let mut signif_y = signif(&abs_bits_y);
        let mut signif_x = signif(&abs_bits_x);
        // Examine magnitude of self / other
        let exp_quot =
            exp_bits(&abs_bits_y) as i32 - exp_bits(&abs_bits_x) as i32;
        const MAX_SHIFT: u32 = EXP_BITS - 1;
        const SHIFT_UPPER_LIMIT: i32 = MAX_SHIFT as i32;
        const SHIFT_LOWER_LIMIT: i32 = -SHIFT_UPPER_LIMIT;
        const UPPER_CUT_OFF: i32 = LARGE_EXP_CUT_OFF as i32;
        match exp_quot {
            0 => {
                signif_y <<= PREC_ADJ;
                signif_x <<= PREC_ADJ;
            }
            sh @ 1..=SHIFT_UPPER_LIMIT => {
                signif_y <<=
                    PREC_ADJ + min(sh.unsigned_abs(), MAX_SHIFT - PREC_ADJ);
                signif_x <<=
                    MAX_SHIFT - max(sh.unsigned_abs(), MAX_SHIFT - PREC_ADJ);
            }
            sh @ SHIFT_LOWER_LIMIT..=-1 => {
                signif_y <<=
                    MAX_SHIFT - max(sh.unsigned_abs(), MAX_SHIFT - PREC_ADJ);
                signif_x <<=
                    PREC_ADJ + min(sh.unsigned_abs(), MAX_SHIFT - PREC_ADJ);
            }
            UPPER_CUT_OFF.. => {
                return FRAC_PI_2.map_atan2_signs(self.sign(), other.sign());
            }
            _ => {
                return self
                    .div(other)
                    .abs()
                    .atan()
                    .map_atan2_signs(self.sign(), other.sign());
            }
        };
        let y = FP248::from(&signif_y);
        let x = FP248::from(&signif_x);
        f256::from(&y.atan2(&x)).map_atan2_signs(self.sign(), other.sign())
    }
}

#[cfg(test)]
mod sin_cos_tests {
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

#[cfg(test)]
mod atan_tests {
    use core::{ops::Neg, str::FromStr};

    use super::*;
    use crate::{
        consts::{FRAC_1_PI, FRAC_PI_3, FRAC_PI_4, FRAC_PI_6},
        EPSILON, ONE_HALF,
    };

    #[test]
    fn test_atan_inf() {
        assert_eq!(f256::INFINITY.atan(), FRAC_PI_2);
    }

    #[test]
    fn test_atan_large_cutoff() {
        let f = f256 {
            bits: LARGE_CUT_OFF,
        };
        assert_eq!(f.atan(), FRAC_PI_2);
    }

    #[test]
    fn test_atan_zero() {
        assert_eq!(f256::ZERO.atan(), f256::ZERO);
        assert_eq!(f256::NEG_ZERO.atan(), f256::ZERO);
    }

    #[test]
    fn test_atan_one() {
        assert_eq!(f256::ONE.atan(), FRAC_PI_4);
        assert_eq!(f256::NEG_ONE.atan(), -FRAC_PI_4);
    }

    #[test]
    fn test_atan_sqrt_3() {
        let t = f256::from(3);
        let mut f = t.sqrt();
        // arctan √3 = ⅓π
        assert_eq!(f.atan(), FRAC_PI_3);
        assert_eq!(f.neg().atan(), -FRAC_PI_3);
        // arctan ⅓√3 = π/6
        f /= t;
        assert_eq!(f.atan(), FRAC_PI_6);
        assert_eq!(f.neg().atan(), -FRAC_PI_6);
    }

    #[test]
    fn test_atan_frac_1_pi() {
        let f1 = FRAC_1_PI.atan();
        let f2 = f256::ONE.atan2(&PI);
        let d = f1 - f2;
        assert!(d.abs() <= EPSILON);
    }

    #[test]
    fn test_atan_frac_pi_2() {
        let s = "1.00388482185388721414842394491713228829210446059487057472971282410801519";
        let a = f256::from_str(s).unwrap();
        let f1 = FRAC_PI_2.atan();
        assert_eq!(f1, a);
        let f2 = PI.atan2(&f256::TWO);
        assert_eq!(f1, f2);
    }

    #[test]
    fn test_atan_frac_5_pi_4() {
        let s = "1.32144796778372235539166569069508390109061014033053361477468861418765787";
        let a = f256::from_str(s).unwrap();
        let f = PI + FRAC_PI_4;
        assert_eq!(f.atan(), a);
        let f = f256::TEN * PI;
        assert_eq!(f.atan2(&f256::from(8_f64)), a);
    }

    #[test]
    fn test_atan_frac_51043_7() {
        let s = "1.570659187521027203661619536335073835579283228441242208112611672132902725";
        let a = f256::from_str(s).unwrap();
        let n = f256::from(51043);
        let d = f256::from(7);
        let f = n / d;
        println!("{}", f.atan());
        // assert_eq!(f.atan(), a);
        assert_eq!(n.atan2(&d), a);
    }
}
