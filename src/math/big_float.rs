// ---------------------------------------------------------------------------
// Copyright:   (c) 2023 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use core::{
    cmp::{max, Ordering},
    mem::swap,
    ops::{Add, AddAssign, Neg, Shr, ShrAssign, Sub, SubAssign},
};

use crate::{
    abs_bits,
    big_uint::{u256, BigUIntHelper},
    exp_bits, f256,
    math::cordic::circular::{cordic_atan, cordic_atan2, cordic_sin_cos},
    norm_bit, signif, EMAX, EMIN, EXP_BIAS, FRACTION_BITS, HI_EXP_MASK,
    HI_FRACTION_BITS, HI_SIGN_SHIFT, SIGNIFICAND_BITS,
};

fn add_signifs(x: &u256, y: &u256) -> (u256, i32) {
    debug_assert!(x.hi.leading_zeros() == 1 || y.hi.leading_zeros() == 1);
    let mut sum = x + y;
    let mut exp_adj = 0;
    if sum.hi.leading_zeros() == 0 {
        // sum.idiv_pow2(1);
        sum >>= 1;
        exp_adj = 1;
    }
    (sum, exp_adj)
}

fn sub_signifs(x: &u256, y: &u256) -> (u256, i32) {
    debug_assert!(x >= y);
    debug_assert!(x.hi.leading_zeros() == 1);
    let mut diff = x - y;
    let shl = diff.leading_zeros() - 1;
    diff <<= shl;
    (diff, -(shl as i32))
}

/// Representation of the number sign * signif * 2^exp.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct BigFloat {
    pub(crate) sign: i32,
    pub(crate) exp: i32,
    // Layout of the 256 bits of the `signif` member: olfff…fff
    // o = reserved bit for overflow handling in addition
    // l = 1 leading bit (always 1 except for FP255::ZERO)
    // f = 254 fractional bits
    pub(crate) signif: u256,
}

pub(crate) const SIGNIF_ONE: u256 = u256 {
    hi: 1_u128 << (BigFloat::FRACTION_BITS - 128),
    lo: 0_u128,
};

impl BigFloat {
    pub(crate) const FRACTION_BITS: u32 = 254;
    pub(crate) const ZERO: Self = Self {
        sign: 0,
        exp: 0,
        signif: u256::ZERO,
    };
    pub(crate) const ONE: Self = Self {
        sign: 1,
        exp: 0,
        signif: SIGNIF_ONE,
    };
    pub(crate) const NEG_ONE: Self = Self {
        sign: -1,
        exp: 0,
        signif: SIGNIF_ONE,
    };
    pub(crate) const EPSILON: Self = Self {
        sign: 1,
        exp: -(Self::FRACTION_BITS as i32),
        signif: SIGNIF_ONE,
    };
    // 3.1415926535897932384626433832795028841971693993751058209749445923078164062862
    pub(crate) const PI: BigFloat = BigFloat {
        sign: 1,
        exp: 1,
        signif: u256::new(
            0x6487ed5110b4611a62633145c06e0e68,
            0x948127044533e63a0105df531d89cd91,
        ),
    };
    // 1.5707963267948966192313216916397514420985846996875529104874722961539082031431
    pub(crate) const FRAC_PI_2: BigFloat = BigFloat {
        sign: 1,
        exp: 0,
        signif: u256::new(
            0x6487ed5110b4611a62633145c06e0e68,
            0x948127044533e63a0105df531d89cd91,
        ),
    };

    // TODO: remove (inline) this fn when trait fns can be constant!
    pub(crate) const fn from_f256(f: &f256) -> Self {
        if f.eq_zero() {
            return Self::ZERO;
        }
        const PREC_ADJ: u32 = BigFloat::FRACTION_BITS - FRACTION_BITS;
        let abs_bits_f = abs_bits(f);
        debug_assert!(abs_bits_f.hi < HI_EXP_MASK); // f is finite?
        debug_assert!(!abs_bits_f.is_zero());
        let signif_f = signif(&abs_bits_f);
        let shl = signif_f.leading_zeros()
            - (u256::BITS - BigFloat::FRACTION_BITS - 1);
        let exp_f = exp_bits(&abs_bits_f) as i32 + 1
            - norm_bit(&abs_bits_f) as i32
            - EXP_BIAS as i32
            - shl as i32
            + PREC_ADJ as i32;
        Self {
            sign: (-1_i32).pow(f.sign()),
            exp: exp_f,
            signif: signif_f.shift_left(shl),
        }
    }

    #[inline(always)]
    pub(crate) const fn is_zero(&self) -> bool {
        self.sign == 0
    }

    #[inline(always)]
    pub(crate) fn flip_sign(&mut self) {
        self.sign *= -1;
    }

    #[inline(always)]
    pub(crate) const fn abs(&self) -> Self {
        Self {
            sign: self.sign.abs(),
            exp: self.exp,
            signif: self.signif,
        }
    }

    fn iadd(&mut self, other: &Self) {
        if self.is_zero() {
            *self = *other;
            return;
        }
        if other.is_zero() {
            return;
        }
        let exp = max(self.exp, other.exp);
        let (mut signif_self, rem_self) =
            self.signif.widening_shr((exp - self.exp) as u32);
        let (mut signif_other, rem_other) =
            other.signif.widening_shr((exp - other.exp) as u32);
        let op =
            [add_signifs, sub_signifs][(self.sign != other.sign) as usize];
        if signif_self < signif_other {
            swap(&mut signif_self, &mut signif_other);
            self.sign = other.sign;
        }
        let mut exp_adj = 0_i32;
        (self.signif, exp_adj) = op(&signif_self, &signif_other);
        if self.signif.is_zero() {
            self.sign = 0;
            self.exp = 0;
        } else {
            self.exp = (exp + exp_adj)
        };
    }

    #[inline(always)]
    fn isub(&mut self, other: &Self) {
        if self == other {
            *self = BigFloat::ZERO;
        } else {
            self.iadd(&other.neg());
        }
    }

    #[inline(always)]
    pub(crate) fn sin_cos(&self) -> (Self, Self) {
        cordic_sin_cos(&self)
    }

    #[inline(always)]
    pub(crate) fn atan(&self) -> Self {
        cordic_atan(*self)
    }

    #[inline(always)]
    pub(crate) fn atan2(&self, other: &Self) -> Self {
        cordic_atan2(&self, other)
    }
}

impl From<&u256> for BigFloat {
    /// Convert a raw u256 into a Float, without any modification, i.e
    /// interptret the given value i as i * 2⁻²⁵⁵
    #[inline(always)]
    fn from(ui: &u256) -> Self {
        Self {
            sign: 1,
            exp: 0,
            signif: *ui,
        }
    }
}

impl From<&f256> for BigFloat {
    #[inline(always)]
    fn from(f: &f256) -> Self {
        Self::from_f256(f)
    }
}

impl From<&BigFloat> for f256 {
    fn from(fp: &BigFloat) -> Self {
        if fp.is_zero() {
            return Self::ZERO;
        }
        const PREC_ADJ: u32 = BigFloat::FRACTION_BITS - FRACTION_BITS;
        const EXP_UNDERFLOW: i32 = EMIN - SIGNIFICAND_BITS as i32;
        const EXP_LOWER_SUBNORMAL: i32 = EXP_UNDERFLOW + 1;
        const EXP_UPPER_SUBNORMAL: i32 = EMIN - 1;
        const EXP_OVERFLOW: i32 = f256::MAX_EXP;
        let mut f256_bits = match fp.exp {
            ..=EXP_UNDERFLOW => u256::ZERO,
            EXP_LOWER_SUBNORMAL..=EXP_UPPER_SUBNORMAL => fp
                .signif
                .div_pow2(PREC_ADJ.saturating_add_signed(EMIN - fp.exp)),
            EMIN..=EMAX => {
                const TIE: u256 = u256::new(1_u128 << 127, 0);
                let (mut bits, rem) = fp.signif.widening_shr(PREC_ADJ);
                // -1 because we add the significand incl. hidden bit.
                let exp_bits = (EXP_BIAS.saturating_add_signed(fp.exp - 1)
                    as u128)
                    << HI_FRACTION_BITS;
                bits.hi += exp_bits;
                // Final rounding. Possibly overflowing into the exponent,
                // but that is ok.
                if rem > TIE || (rem == TIE && (bits.lo & 1) == 1) {
                    bits.incr();
                }
                bits
            }
            EXP_OVERFLOW.. => f256::INFINITY.bits,
        };
        f256_bits.hi |= ((fp.sign < 0) as u128) << HI_SIGN_SHIFT;
        f256 { bits: f256_bits }
    }
}

#[cfg(test)]
mod from_into_f256_tests {
    use super::*;

    fn assert_normal_eq(f: &f256, g: &BigFloat) {
        const PREC_DIFF: u32 = BigFloat::FRACTION_BITS - FRACTION_BITS;
        debug_assert!(f.is_normal());
        assert_eq!((-1_i32).pow(f.sign()), g.sign);
        assert_eq!(f.exponent() + FRACTION_BITS as i32, g.exp);
        assert_eq!(&f.significand() << PREC_DIFF, g.signif)
    }

    #[test]
    fn test_neg_one() {
        let fp = BigFloat::from(&f256::NEG_ONE);
        assert_eq!(fp, BigFloat::NEG_ONE);
        let f = f256::from(&fp);
        assert_normal_eq(&f, &fp);
    }

    #[test]
    fn test_normal_gt_one() {
        let f = f256::from(1.5);
        let fp = BigFloat::from(&f);
        assert_normal_eq(&f, &fp);
        let f = f256::from(&fp);
        assert_normal_eq(&f, &fp);
    }

    #[test]
    fn test_normal_lt_one() {
        let f = f256::from(0.625);
        let fp = BigFloat::from(&f);
        assert_normal_eq(&f, &fp);
        let f = f256::from(&fp);
        assert_normal_eq(&f, &fp);
    }

    #[test]
    fn test_normal_lt_minus_one() {
        let f = f256::from(-7.5);
        let fp = BigFloat::from(&f);
        assert_normal_eq(&f, &fp);
        let f = f256::from(&fp);
        assert_normal_eq(&f, &fp);
    }

    #[test]
    fn test_min_f256() {
        let f = f256::MIN;
        let fp = BigFloat::from(&f);
        assert_normal_eq(&f, &fp);
        let f = f256::from(&fp);
        assert_normal_eq(&f, &fp);
    }

    #[test]
    fn test_epsilon() {
        let f = f256::EPSILON;
        let fp = BigFloat::from(&f);
        assert_normal_eq(&f, &fp);
        let f = f256::from(&fp);
        assert_normal_eq(&f, &fp);
    }

    #[test]
    fn test_min_gt_zero() {
        let f = f256::MIN_GT_ZERO;
        let fp = BigFloat::from(&f);
        assert_eq!((-1_i32).pow(f.sign()), fp.sign);
        assert_eq!(f.exponent(), fp.exp);
        assert_eq!(&f.significand() << BigFloat::FRACTION_BITS, fp.signif);
        let f = f256::from(&fp);
        assert_eq!((-1_i32).pow(f.sign()), fp.sign);
        assert_eq!(f.exponent(), fp.exp);
        assert_eq!(&f.significand() << BigFloat::FRACTION_BITS, fp.signif)
    }
}

#[cfg(test)]
mod into_f256_tests {
    use super::*;
    use crate::consts::PI;

    #[test]
    fn test_overflow_1() {
        let fp = BigFloat {
            sign: -1,
            exp: f256::MAX_EXP,
            signif: BigFloat::ONE.signif,
        };
        let f = f256::from(&fp);
        assert_eq!(f, f256::NEG_INFINITY);
    }

    #[test]
    fn test_overflow_2() {
        let fp = BigFloat {
            sign: 1,
            exp: EMAX,
            signif: u256::new(u128::MAX >> 1, u128::MAX - 7),
        };
        let f = f256::from(&fp);
        assert_eq!(f, f256::INFINITY);
    }

    #[test]
    fn test_overflow_3() {
        let sh = BigFloat::FRACTION_BITS - FRACTION_BITS - 1;
        let fp = BigFloat {
            sign: 1,
            exp: 0,
            signif: u256::new(u128::MAX >> 1, (u128::MAX >> sh) << sh),
        };
        let f = f256::from(&fp);
        assert_eq!(f, f256::TWO);
    }

    #[test]
    fn test_underflow() {
        let fp = BigFloat {
            sign: 1,
            exp: EMIN - SIGNIFICAND_BITS as i32,
            signif: u256::new(1_u128 << 127, 0_u128),
        };
        let f = f256::from(&fp);
        assert_eq!(f, f256::ZERO);
    }

    #[test]
    fn test_round_to_epsilon() {
        let fp = BigFloat {
            sign: 1,
            exp: -237,
            signif: u256::new(u128::MAX >> 1, u128::MAX),
        };
        let f = f256::from(&fp);
        assert_eq!(f, f256::EPSILON);
    }

    #[test]
    fn test_f256_pi() {
        let f = PI;
        let fp = BigFloat::from(&f);
        let g = f256::from(&fp);
        assert_eq!(f, g);
    }
}

impl Neg for BigFloat {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::Output {
            sign: self.sign * -1,
            exp: self.exp,
            signif: self.signif,
        }
    }
}

impl PartialOrd for BigFloat {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some((self.sign, self.exp, self.signif).cmp(&(
            other.sign,
            other.exp,
            other.signif,
        )))
    }
}

impl Shr<u32> for &BigFloat {
    type Output = BigFloat;

    fn shr(self, rhs: u32) -> Self::Output {
        let exp_adj = [rhs as i32, 0][self.signif.is_zero() as usize];
        Self::Output {
            sign: self.sign,
            exp: self.exp - exp_adj,
            signif: self.signif,
        }
    }
}

impl ShrAssign<u32> for BigFloat {
    fn shr_assign(&mut self, rhs: u32) {
        self.exp -= [rhs as i32, 0][self.signif.is_zero() as usize];
    }
}

impl AddAssign<&Self> for BigFloat {
    #[inline(always)]
    fn add_assign(&mut self, rhs: &Self) {
        self.iadd(rhs);
    }
}

impl SubAssign<&Self> for BigFloat {
    fn sub_assign(&mut self, rhs: &Self) {
        self.isub(rhs);
    }
}

#[cfg(test)]
mod add_sub_tests {
    use super::*;

    #[test]
    fn test_add_same_sign() {
        let mut f = BigFloat::ONE;
        f += &BigFloat::ONE;
        assert_eq!(f.sign, 1);
        assert_eq!(f.exp, 1);
        assert_eq!(f.signif, BigFloat::ONE.signif);
        f.flip_sign();
        f += &f.clone();
        assert_eq!(f.sign, -1);
        assert_eq!(f.exp, 2);
        assert_eq!(f.signif, BigFloat::ONE.signif);
    }

    #[test]
    fn test_add_diff_sign() {
        let mut f = BigFloat::ONE;
        f += &BigFloat::NEG_ONE;
        assert_eq!(f, BigFloat::ZERO);
        let mut f = BigFloat {
            sign: -1,
            exp: 0,
            signif: u256::new(BigFloat::ONE.signif.hi, 1),
        };
        f += &BigFloat::EPSILON;
        assert_eq!(f, BigFloat::NEG_ONE);
        let mut g = BigFloat::EPSILON;
        g += &f.clone();
        assert_eq!(g.sign, -1);
        assert_eq!(g.exp, -1);
        assert_eq!(g.signif, u256::new(u128::MAX >> 1, u128::MAX - 1));
    }

    #[test]
    fn test_add_small_values() {
        let mut a = BigFloat {
            sign: 1,
            exp: -31,
            signif: u256::new(
                0x762786250e76f22407ff39555489fe14,
                0xc7367a8a3b4fb9bb64012ff173ba3820,
            ),
        };
        let b = BigFloat {
            sign: -1,
            exp: -29,
            signif: u256::new(
                0x4a861bd3d04784350b3364f35c90403f,
                0xbc67c2e66a540f53f03c854744f355a4,
            ),
        };
        let d = BigFloat {
            sign: -1,
            exp: -30,
            signif: u256::new(
                0x59f8749519538f5812672d3c0edb8175,
                0x15344887b70041ca2e787295d0098f38,
            ),
        };
        a += &b;
        assert_eq!(a, d);
    }

    #[test]
    fn test_sub_diff_sign() {
        let mut f = BigFloat::NEG_ONE;
        f -= &BigFloat::ONE;
        assert_eq!(f.sign, -1);
        assert_eq!(f.exp, 1);
        assert_eq!(f.signif, BigFloat::ONE.signif);
        let mut g = f;
        g.flip_sign();
        f -= &g;
        assert_eq!(f.sign, -1);
        assert_eq!(f.exp, 2);
        assert_eq!(f.signif, BigFloat::ONE.signif);
    }

    #[test]
    fn test_sub_same_sign() {
        let mut f = BigFloat::NEG_ONE;
        f -= &BigFloat::NEG_ONE;
        assert_eq!(f, BigFloat::ZERO);
        let mut f = BigFloat {
            sign: 1,
            exp: 0,
            signif: u256::new(BigFloat::ONE.signif.hi, 1),
        };
        f -= &BigFloat::EPSILON;
        assert_eq!(f, BigFloat::ONE);
        let mut g = BigFloat::EPSILON;
        g -= &f.clone();
        assert_eq!(g.sign, -1);
        assert_eq!(g.exp, -1);
        assert_eq!(g.signif, u256::new(u128::MAX >> 1, u128::MAX - 1));
    }

    #[test]
    fn test_sub_small_value() {
        let mut a = BigFloat {
            sign: 1,
            exp: -1,
            signif: u256::new(
                0x6487ed5110b4611a62633145c06e0e68,
                0x033cee6a628198d4c2836363a132d844,
            ),
        };
        let b = BigFloat {
            sign: -1,
            exp: -128,
            signif: u256::new(
                0x40000000000000000000000000000000,
                0x00000000000000000000000000000000,
            ),
        };
        let d = BigFloat {
            sign: 1,
            exp: -1,
            signif: u256::new(
                0x6487ed5110b4611a62633145c06e0e68,
                0x833cee6a628198d4c2836363a132d844,
            ),
        };
        a -= &b;
        assert_eq!(a, d);
    }
}
