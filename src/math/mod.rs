// ---------------------------------------------------------------------------
// Copyright:   (c) 2023 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

mod circular_fns;
mod cordic;
mod sqrt;

use std::{
    cmp::{max, Ordering},
    ops::{AddAssign, Neg, Shr, ShrAssign, SubAssign},
};

use cordic::circular::{cordic_atan, cordic_sin_cos};

use crate::{
    big_uint::u256, f256, split_f256_enc, EXP_BIAS, EXP_BITS, FRACTION_BITS,
    HI_FRACTION_BITS, SIGNIFICAND_BITS,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct FP248 {
    sign: u32,
    signif: u256,
}

impl FP248 {
    const FRACTION_BITS: u32 = 248;
    const ZERO: Self = Self {
        sign: 0,
        signif: u256::ZERO,
    };
    const NEG_ZERO: Self = Self {
        sign: 1,
        signif: u256::ZERO,
    };
    const ONE: Self = Self {
        sign: 0,
        signif: u256::new(1_u128 << (FP248::FRACTION_BITS - 128), 0_u128),
    };
    const NEG_ONE: Self = Self {
        sign: 1,
        signif: u256::new(1_u128 << (FP248::FRACTION_BITS - 128), 0_u128),
    };
    const EPSILON: Self = Self {
        sign: 0,
        signif: u256::new(0_u128, 1_u128),
    };
    const MAX: Self = Self {
        sign: 0,
        signif: u256::MAX,
    };
    const MIN: Self = Self {
        sign: 1,
        signif: u256::MAX,
    };
    // 1.5707963267948966192313216916397514420985846996875529104874722961539082
    const FRAC_PI_2: FP248 = FP248 {
        sign: 0,
        signif: u256::new(
            0x01921fb54442d18469898cc51701b839,
            0xa252049c1114cf98e804177d4c762000,
        ),
    };

    #[inline(always)]
    fn flip_sign(&mut self) {
        self.sign ^= 1;
    }

    fn sin_cos(&self, quadrant: u32) -> (Self, Self) {
        let (mut sin, mut cos) = cordic_sin_cos(&self);
        // Map result according to quadrant
        match quadrant {
            0 => (sin, cos),
            1 => {
                &sin.flip_sign();
                (cos, sin)
            }
            2 => {
                &sin.flip_sign();
                &cos.flip_sign();
                (sin, cos)
            }
            3 => {
                &cos.flip_sign();
                (cos, sin)
            }
            _ => unreachable!(),
        }
    }

    #[inline(always)]
    fn atan(&self) -> Self {
        cordic_atan(&self)
    }
}

impl From<&f256> for FP248 {
    fn from(f: &f256) -> Self {
        debug_assert!(f.is_finite());
        if f.eq_zero() {
            return FP248::ZERO;
        }
        let (sign, exp, mut signif) = split_f256_enc(&f);
        let shl = max(0, FP248::FRACTION_BITS as i32 + exp) as u32;
        debug_assert!(shl <= EXP_BITS, "Value too large!");
        let shr = max(0, -exp - FP248::FRACTION_BITS as i32) as u32;
        signif <<= shl;
        signif >>= shr;
        Self { sign, signif }
    }
}

#[cfg(test)]
mod from_f256_tests {
    use super::*;

    #[test]
    fn test_normal_gt_one() {
        let f = f256::from(1.5);
        let fp = FP248::from(&f);
        assert_eq!(fp.sign, 0);
        assert_eq!(fp.signif, u256::new(3_u128 << 119, 0));
    }

    #[test]
    fn test_normal_lt_one() {
        let f = f256::from(0.625);
        let fp = FP248::from(&f);
        assert_eq!(fp.sign, 0);
        assert_eq!(fp.signif, u256::new(5_u128 << 117, 0));
    }

    #[test]
    fn test_normal_lt_minus_one() {
        let f = f256::from(-7.5);
        let fp = FP248::from(&f);
        assert_eq!(fp.sign, 1);
        assert_eq!(fp.signif, u256::new(15_u128 << 119, 0));
    }

    #[test]
    fn test_normal_lt_zero() {
        let f = f256::from(-0.5);
        let fp = FP248::from(&f);
        assert_eq!(fp.sign, 1);
        assert_eq!(fp.signif, u256::new(1_u128 << 119, 0));
    }
}

impl From<&FP248> for f256 {
    fn from(value: &FP248) -> Self {
        const PREC_DIFF: u32 = FP248::FRACTION_BITS - FRACTION_BITS;
        let mut c = value.signif;
        let mut t = EXP_BIAS;
        // Normalize significand
        let nlz = c.leading_zeros();
        match nlz.cmp(&EXP_BITS) {
            Ordering::Greater => {
                // Compensate prec diff
                c.idiv_pow2(PREC_DIFF);
                // Shift left. Rounding may have added a bit, so need to check
                // leading zeroes again.
                let shift = (c.leading_zeros() - EXP_BITS);
                c <<= shift;
                t -= shift;
            }
            Ordering::Less => {
                // Shift right and round.
                let shift = (EXP_BITS - nlz);
                t = EXP_BIAS + shift - PREC_DIFF;
                c.idiv_pow2(shift);
                // Rounding may have caused significand to overflow.
                if (c.hi >> (HI_FRACTION_BITS + 1)) != 0 {
                    t += 1;
                    c >>= 1;
                }
            }
            _ => {}
        }
        debug_assert!(c.is_zero() || c.leading_zeros() == EXP_BITS);
        f256::new(
            c,
            t * !c.is_zero() as u32,
            value.sign * !c.is_zero() as u32,
        )
    }
}

#[cfg(test)]
mod into_f256_tests {
    use core::str::FromStr;

    use super::*;
    use crate::consts::PI;

    #[test]
    fn test_lt_1() {
        let fp = FP248 {
            sign: 1,
            signif: u256::new(5_u128 << 117, 0),
        };
        let f = f256::from(&fp);
        assert_eq!(f, f256::from_str("-0.625").unwrap());
    }

    #[test]
    fn test_gt_1_lt_2() {
        let fp = FP248 {
            sign: 0,
            signif: u256::new(10_u128 << 117, 0),
        };
        let f = f256::from(&fp);
        assert_eq!(f, f256::from_str("1.25").unwrap());
    }

    #[test]
    fn test_gt_2() {
        let fp = FP248 {
            sign: 0,
            signif: u256::new(13_u128 << 119, 0),
        };
        let f = f256::from(&fp);
        assert_eq!(f, f256::from_str("6.5").unwrap());
    }

    #[test]
    fn test_epsilon() {
        let f = f256::EPSILON;
        let fp = FP248::from(&f);
        let g = f256::from(&fp);
        assert_eq!(f, g);
    }

    #[test]
    fn test_gt_epsilon() {
        let fp = FP248 {
            sign: 0,
            signif: u256::new(0, 3 << 11),
        };
        let f = f256::from(&fp);
        assert_eq!(f, f256::TWO * f256::EPSILON);
    }

    #[test]
    fn test_round_to_epsilon() {
        let fp = FP248 {
            sign: 0,
            signif: u256::new(0, 3 << 10),
        };
        let f = f256::from(&fp);
        assert_eq!(f, f256::EPSILON);
    }

    #[test]
    fn test_round_to_zero() {
        let fp = FP248 {
            sign: 0,
            signif: u256::new(0, 1 << 11),
        };
        let f = f256::from(&fp);
        assert_eq!(f, f256::ZERO);
    }

    #[test]
    fn test_max() {
        let f = f256::from(256) - f256::from(1e-66);
        let fp = FP248::from(&f);
        let g = f256::from(&fp);
        assert_eq!(f, g);
    }

    #[test]
    fn test_f256_pi() {
        let f = PI;
        let fp = FP248::from(&f);
        let g = f256::from(&fp);
        assert_eq!(f, g);
    }
}

impl Neg for FP248 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::Output {
            sign: self.sign ^ 1,
            signif: self.signif,
        }
    }
}

impl PartialOrd for FP248 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(
            (self.sign ^ 1, self.signif).cmp(&(other.sign ^ 1, other.signif)),
        )
    }
}

impl Shr<u32> for &FP248 {
    type Output = FP248;

    fn shr(self, rhs: u32) -> Self::Output {
        Self::Output {
            sign: self.sign,
            signif: &self.signif >> rhs,
        }
    }
}

impl ShrAssign<u32> for FP248 {
    fn shr_assign(&mut self, rhs: u32) {
        self.signif >>= rhs;
    }
}

impl AddAssign<&Self> for FP248 {
    fn add_assign(&mut self, rhs: &Self) {
        if self.sign == rhs.sign {
            self.signif += &rhs.signif;
        } else {
            if self.signif >= rhs.signif {
                self.signif -= &rhs.signif;
            } else {
                self.sign = rhs.sign;
                self.signif = &rhs.signif - &self.signif;
            }
        }
    }
}

impl SubAssign<&Self> for FP248 {
    fn sub_assign(&mut self, rhs: &Self) {
        if self.sign != rhs.sign {
            self.signif += &rhs.signif;
        } else {
            if self.signif >= rhs.signif {
                self.signif -= &rhs.signif;
            } else {
                self.signif = &rhs.signif - &self.signif;
                self.flip_sign();
            }
        }
    }
}

#[cfg(test)]
mod add_sub_tests {
    use super::*;

    #[test]
    fn test_add_same_sign() {
        let mut f = FP248::ONE;
        f += &FP248::ONE;
        assert_eq!(f.sign, 0);
        assert_eq!(
            f.signif,
            u256::new(2_u128 << (FP248::FRACTION_BITS - 128), 0)
        );
        f.flip_sign();
        f += &f.clone();
        assert_eq!(f.sign, 1);
        assert_eq!(
            f.signif,
            u256::new(4_u128 << (FP248::FRACTION_BITS - 128), 0)
        );
    }

    #[test]
    fn test_add_diff_sign() {
        let mut f = FP248::ONE;
        f += &FP248::NEG_ONE;
        assert_eq!(f, FP248::ZERO);
        let mut f = FP248 {
            sign: 1,
            signif: u256::new(1, 1),
        };
        f += &FP248::EPSILON;
        assert_eq!(f.sign, 1);
        assert_eq!(f.signif, u256::new(1, 0));
        let mut g = FP248::EPSILON;
        g += &f.clone();
        assert_eq!(g.sign, 1);
        assert_eq!(g.signif, u256::new(0, u128::MAX));
    }

    #[test]
    fn test_sub_diff_sign() {
        let mut f = FP248::NEG_ONE;
        f -= &FP248::ONE;
        assert_eq!(f.sign, 1);
        assert_eq!(
            f.signif,
            u256::new(2_u128 << (FP248::FRACTION_BITS - 128), 0)
        );
        let mut g = f;
        g.flip_sign();
        f -= &g;
        assert_eq!(f.sign, 1);
        assert_eq!(
            f.signif,
            u256::new(4_u128 << (FP248::FRACTION_BITS - 128), 0)
        );
    }

    #[test]
    fn test_sub_same_sign() {
        let mut f = FP248::NEG_ONE;
        f -= &FP248::NEG_ONE;
        assert_eq!(f, FP248::NEG_ZERO);
        let mut f = FP248 {
            sign: 0,
            signif: u256::new(1, 1),
        };
        f -= &FP248::EPSILON;
        assert_eq!(f.sign, 0);
        assert_eq!(f.signif, u256::new(1, 0));
        let mut g = FP248::EPSILON;
        g -= &f.clone();
        assert_eq!(g.sign, 1);
        assert_eq!(g.signif, u256::new(0, u128::MAX));
    }
}
