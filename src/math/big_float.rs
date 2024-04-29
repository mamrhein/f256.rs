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
    ops::{
        Add, AddAssign, Mul, MulAssign, Neg, Shr, ShrAssign, Sub, SubAssign,
    },
};

use crate::{
    abs_bits,
    big_uint::{u256, u512, BigUIntHelper},
    exp_bits, f256, norm_bit, signif, EMAX, EMIN, EXP_BIAS, FRACTION_BITS,
    HI_EXP_MASK, HI_FRACTION_BITS, HI_SIGN_SHIFT, SIGNIFICAND_BITS,
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

fn mul_signifs(x: &u256, y: &u256) -> (u512, i32) {
    debug_assert!(x.hi.leading_zeros() == 1);
    debug_assert!(y.hi.leading_zeros() == 1);
    let (lo, hi) = x.widening_mul(y);
    let nlz = hi.leading_zeros();
    let res = &u512 { hi, lo } << (nlz - 1);
    (res, (nlz == 2) as i32)
}

/// Representation of the number sign * signif * 2^(exp-254).
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct BigFloat {
    pub(crate) sign: i32,
    pub(crate) exp: i32,
    // Layout of the 256 bits of the `signif` member: olfff…fff
    // o = reserved bit for overflow handling in addition
    // l = 1 leading bit (always 1 except for BigFloat::ZERO)
    // f = 254 fractional bits
    pub(crate) signif: u256,
}

const SIGNIF_ONE: u256 = u256 {
    hi: 1_u128 << (BigFloat::FRACTION_BITS - 128),
    lo: 0_u128,
};

const TIE: u256 = u256 {
    hi: 1_u128 << 127,
    lo: 0_u128,
};

impl BigFloat {
    pub(crate) const FRACTION_BITS: u32 = 254;
    pub(crate) const ZERO: Self = Self {
        sign: 0,
        signif: u256::ZERO,
        exp: 0,
    };
    pub(crate) const ONE: Self = Self {
        sign: 1,
        signif: SIGNIF_ONE,
        exp: 0,
    };
    pub(crate) const NEG_ONE: Self = Self {
        sign: -1,
        signif: SIGNIF_ONE,
        exp: 0,
    };
    pub(crate) const ONE_HALF: Self = Self {
        sign: 1,
        signif: SIGNIF_ONE,
        exp: -1,
    };
    // 2^-254
    pub(crate) const EPSILON: Self = Self {
        sign: 1,
        signif: SIGNIF_ONE,
        exp: -(Self::FRACTION_BITS as i32),
    };
    // PI = ◯₂₅₅(π) =
    // 3.1415926535897932384626433832795028841971693993751058209749445923078164062862
    pub(crate) const PI: BigFloat = BigFloat::new(
        0x6487ed5110b4611a62633145c06e0e68,
        0x948127044533e63a0105df531d89cd91,
        1,
    );
    // FRAC_PI_2 = ◯₂₅₅(½π) =
    // 1.5707963267948966192313216916397514420985846996875529104874722961539082031431
    pub(crate) const FRAC_PI_2: BigFloat = BigFloat::new(
        0x6487ed5110b4611a62633145c06e0e68,
        0x948127044533e63a0105df531d89cd91,
        0,
    );
    // FRAC_PI_4 = ◯₂₅₅(½π) =
    // 1.5707963267948966192313216916397514420985846996875529104874722961539082031431
    pub(crate) const FRAC_PI_4: BigFloat = BigFloat::new(
        0x6487ed5110b4611a62633145c06e0e68,
        0x948127044533e63a0105df531d89cd91,
        -1,
    );
    // FRAC_3_PI_2 = ◯₂₅₅(3⋅½π) =
    // 4.7123889803846898576939650749192543262957540990626587314624168884617246094293
    pub(crate) const FRAC_3_PI_2: BigFloat = BigFloat::new(
        0x4b65f1fccc8748d3c9ca64f450528ace,
        0x6f60dd4333e6ecab80c4677e56275a2d,
        2,
    );
    // TAU = ◯₂₅₅(2⋅π) =
    // 6.2831853071795864769252867665590057683943387987502116419498891846156328125724
    pub(crate) const TAU: BigFloat = BigFloat::new(
        0x6487ed5110b4611a62633145c06e0e68,
        0x948127044533e63a0105df531d89cd91,
        2,
    );
    // FRAC_3_PI_4 = ◯₂₅₅(3⋅¼π) =
    // 2.3561944901923449288469825374596271631478770495313293657312084442308623047147
    pub(crate) const FRAC_3_PI_4: BigFloat = BigFloat::new(
        0x4b65f1fccc8748d3c9ca64f450528ace,
        0x6f60dd4333e6ecab80c4677e56275a2d,
        1,
    );
    // FRAC_5_PI_4 = ◯₂₅₅(5⋅¼π) =
    // 3.9269908169872415480783042290993786052464617492188822762186807403847705078577
    pub(crate) const FRAC_5_PI_4: BigFloat = BigFloat::new(
        0x7da9e8a554e17960fafbfd9730899202,
        0xb9a170c55680dfc881475727e4ec40f5,
        1,
    );
    // FRAC_7_PI_4 = ◯₂₅₅(7⋅¼π) =
    // 5.4977871437821381673096259207391300473450464489064351867061530365386787110009
    pub(crate) const FRAC_7_PI_4: BigFloat = BigFloat::new(
        0x57f6efa6ee9dd4f71616cb1d08604c9b,
        0x81f10223bc8d6972c0e52368b9d893df,
        2,
    );
    // FRAC_9_PI_4 = ◯₂₅₅(9⋅¼π) =
    // 7.0685834705770347865409476123788814894436311485939880971936253326925869141439
    pub(crate) const FRAC_9_PI_4: BigFloat = BigFloat::new(
        0x7118eafb32caed3daeaf976e787bd035,
        0xa7114be4cdda630141269b3d813b0743,
        2,
    );

    #[inline]
    pub(crate) const fn new(
        signif_hi: i128,
        signif_lo: u128,
        exp: i32,
    ) -> Self {
        debug_assert!(signif_hi.abs().leading_zeros() == 1);
        Self {
            sign: signif_hi.signum() as i32,
            exp,
            signif: u256::new(signif_hi.unsigned_abs(), signif_lo),
        }
    }

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
    pub(crate) const fn quantum(&self) -> Self {
        Self {
            sign: 1,
            exp: self.exp - Self::FRACTION_BITS as i32,
            signif: SIGNIF_ONE,
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
    pub(crate) fn copy_sign(&mut self, other: &Self) {
        self.sign = other.sign;
    }

    #[inline(always)]
    pub(crate) const fn abs(&self) -> Self {
        Self {
            sign: self.sign.abs(),
            exp: self.exp,
            signif: self.signif,
        }
    }

    pub(crate) fn trunc(&self) -> Self {
        let sh = max(Self::FRACTION_BITS as i32 - self.exp, 0) as u32;
        Self {
            sign: self.sign,
            exp: self.exp,
            signif: &(&self.signif >> sh) << sh,
        }
    }

    fn iadd(&mut self, other: &Self) {
        let exp = max(self.exp, other.exp);
        if self.is_zero() || (exp - self.exp) > Self::FRACTION_BITS as i32 {
            *self = *other;
            return;
        }
        if other.is_zero() || (exp - other.exp) > Self::FRACTION_BITS as i32 {
            return;
        }
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

    fn isub(&mut self, other: &Self) {
        if self == other {
            *self = BigFloat::ZERO;
        } else {
            self.iadd(&other.neg());
        }
    }

    fn imul(&mut self, other: &Self) {
        self.sign *= other.sign;
        if self.sign == 0 {
            *self = Self::ZERO;
        } else {
            let (mut prod_signif, exp_adj) =
                mul_signifs(&self.signif, &other.signif);
            // round significand of product to 255 bits
            let rnd = prod_signif.lo > TIE
                || (prod_signif.lo == TIE && prod_signif.hi.is_odd());
            prod_signif.hi += rnd as u128;
            prod_signif.hi >>= (prod_signif.hi.leading_zeros() == 0) as u32;
            self.signif = prod_signif.hi;
            self.exp += other.exp + exp_adj;
        }
    }

    pub(crate) fn imul_add(&mut self, f: &Self, a: &Self) {
        if self.is_zero() || f.is_zero() {
            *self = *a;
            return;
        }
        if a.is_zero() {
            self.imul(f);
            return;
        }
        let mut prod_exp = self.exp + f.exp;
        let mut exp_diff = prod_exp - a.exp;
        const UPPER_LIM_PROD_TOO_SMALL: i32 =
            -(BigFloat::FRACTION_BITS as i32) - 2;
        const LOWER_LIM_ADDEND_TOO_SMALL: i32 =
            2 * BigFloat::FRACTION_BITS as i32 + 3;
        match exp_diff {
            i32::MIN..=UPPER_LIM_PROD_TOO_SMALL => {
                *self = *a;
            }
            LOWER_LIM_ADDEND_TOO_SMALL..=i32::MAX => {
                self.imul(f);
            }
            _ => {
                // -256 < exp_diff < 511
                let prod_sign = self.sign * f.sign;
                let (mut prod_signif, exp_adj) =
                    mul_signifs(&self.signif, &f.signif);
                prod_exp += exp_adj;
                exp_diff += exp_adj;
                let mut addend_signif = u512 {
                    hi: a.signif,
                    lo: u256::ZERO,
                };
                let (x_sign, mut x_signif, mut x_exp, y_signif) =
                    match exp_diff {
                        0 => {
                            // exponents equal => check significands
                            if prod_signif >= addend_signif {
                                // |prod| >= |addend|
                                (
                                    prod_sign,
                                    prod_signif,
                                    prod_exp,
                                    addend_signif,
                                )
                            } else {
                                // |addend| > |prod|
                                (a.sign, addend_signif, a.exp, prod_signif)
                            }
                        }
                        1..=i32::MAX => {
                            // |prod| > |addend|
                            let (q, r) =
                                addend_signif.widening_shr(exp_diff as u32);
                            addend_signif = q;
                            addend_signif.lo.lo |= (r != u512::ZERO) as u128;
                            (prod_sign, prod_signif, prod_exp, addend_signif)
                        }
                        i32::MIN..=-1 => {
                            // |addend| > |prod|
                            let (q, r) = prod_signif
                                .widening_shr(exp_diff.unsigned_abs());
                            prod_signif = q;
                            prod_signif.lo.lo |= (r != u512::ZERO) as u128;
                            (a.sign, addend_signif, a.exp, prod_signif)
                        }
                    };
                self.sign = x_sign;
                self.exp = x_exp;
                if prod_sign == a.sign {
                    x_signif += &y_signif;
                    // addition may have overflowed
                    let mut shr = (x_signif.leading_zeros() == 0) as u32;
                    self.exp += shr as i32;
                    let mut rnd = (x_signif.hi.lo & shr as u128) == 1;
                    x_signif.hi >>= shr;
                    rnd &= (x_signif.lo != u256::ZERO)
                        || (x_signif.lo == u256::ZERO
                            && x_signif.hi.is_odd());
                    rnd |= (x_signif.lo > TIE)
                        || (x_signif.lo == TIE && x_signif.hi.is_odd());
                    x_signif.hi += rnd as u128;
                    shr = (x_signif.hi.leading_zeros() == 0) as u32;
                    self.exp += shr as i32;
                    x_signif.hi >>= shr;
                    self.signif = x_signif.hi;
                } else {
                    x_signif -= &y_signif;
                    // subtraction may have cancelled some or all leading bits
                    let shl = x_signif.leading_zeros() - 1;
                    self.exp -= shl as i32;
                    match shl {
                        0..=256 => {
                            // shifting left by shl bits and then rounding the
                            // low 256 bits => shift right and round by
                            // 256 - shl bits
                            x_signif.idiv_pow2(u256::BITS - shl);
                            self.signif = x_signif.lo;
                        }
                        257..=510 => {
                            // less than 255 bits left => shift left, no
                            // rounding
                            self.signif = &x_signif.lo << shl - 256;
                        }
                        _ => {
                            // all bits cancelled => result is zero
                            *self = BigFloat::ZERO;
                        }
                    }
                };
            }
        }
    }

    #[inline]
    pub(crate) fn mul_add(self, f: &Self, a: &Self) -> Self {
        let mut res = self;
        res.imul_add(f, a);
        res
    }

    /// Computes the rounded sum of two BigFloat values and the remainder.
    #[inline]
    pub(crate) fn sum_exact(&self, rhs: &Self) -> (Self, Self) {
        let s = *self + rhs;
        let d = s - rhs;
        let t1 = *self - &d;
        let t2 = *rhs - &(s - &d);
        let r = t1 + &t2;
        (s, r)
    }

    /// Computes the rounded product of two BigFloat values and the remainder.
    #[inline]
    pub(crate) fn mul_exact(&self, rhs: &Self) -> (Self, Self) {
        let p = *self * rhs;
        let r = self.mul_add(rhs, &p.neg());
        (p, r)
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
        assert_eq!(f.quantum_exponent() + FRACTION_BITS as i32, g.exp);
        assert_eq!(&f.integral_significand() << PREC_DIFF, g.signif)
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
        assert_eq!(f.quantum_exponent(), fp.exp);
        assert_eq!(
            &f.integral_significand() << BigFloat::FRACTION_BITS,
            fp.signif
        );
        let f = f256::from(&fp);
        assert_eq!((-1_i32).pow(f.sign()), fp.sign);
        assert_eq!(f.quantum_exponent(), fp.exp);
        assert_eq!(
            &f.integral_significand() << BigFloat::FRACTION_BITS,
            fp.signif
        )
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

    #[inline]
    fn neg(self) -> Self::Output {
        Self::Output {
            sign: self.sign * -1,
            exp: self.exp,
            signif: self.signif,
        }
    }
}

impl PartialOrd for BigFloat {
    #[inline]
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

impl Add<&Self> for BigFloat {
    type Output = Self;

    fn add(self, rhs: &Self) -> Self::Output {
        let mut res = self;
        res += rhs;
        res
    }
}

impl SubAssign<&Self> for BigFloat {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: &Self) {
        self.isub(rhs);
    }
}

impl Sub<&Self> for BigFloat {
    type Output = Self;

    fn sub(self, rhs: &Self) -> Self::Output {
        let mut res = self;
        res -= rhs;
        res
    }
}

impl MulAssign<&Self> for BigFloat {
    #[inline(always)]
    fn mul_assign(&mut self, rhs: &Self) {
        self.imul(rhs);
    }
}

impl Mul<&Self> for BigFloat {
    type Output = Self;

    fn mul(self, rhs: &Self) -> Self::Output {
        let mut res = self;
        res *= rhs;
        res
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

    #[test]
    fn test_sub_very_small_value() {
        let mut a = BigFloat {
            sign: 1,
            exp: 7,
            signif: u256::new(
                0x6487ed5110b4611a62633145c06e0e68,
                0x033cee6a628198d4c2836363a132d844,
            ),
        };
        let b = BigFloat {
            sign: 1,
            exp: -248,
            signif: u256::new(
                0x40000000000000000000000000000000,
                0x00000000000000000000000000000000,
            ),
        };
        assert_eq!(a - &b, a);
    }
}

#[cfg(test)]
mod mul_tests {
    use super::*;

    #[test]
    fn test_imul_same_sign() {
        let mut x = BigFloat::NEG_ONE;
        x += &BigFloat::EPSILON;
        let y = x;
        x.imul(&y);
        assert_eq!(x.sign, 1);
        assert_eq!(x.exp, -1);
        assert_eq!(x.signif, u256::new(y.signif.hi, y.signif.lo - 2));
    }

    #[test]
    fn test_imul_diff_sign() {
        let mut x = BigFloat::FRAC_PI_2;
        let y = -BigFloat::PI;
        x.imul(&y);
        assert_eq!(x.sign, -1);
        assert_eq!(x.exp, 2);
        assert_eq!(
            x.signif,
            u256::new(
                0x4ef4f326f91779692b71366cc0460d63,
                0x842b351ff06851143341108129e39b47
            )
        );
    }

    #[test]
    fn test_imul_add() {
        let mut x = BigFloat::PI;
        let y = BigFloat {
            sign: 1,
            exp: -1,
            signif: u256::new(
                0x7fffffffffffffffffffffffffffffff,
                0xffffffffffffffffffffffffffffc000,
            ),
        };
        let mut p = x;
        p.imul(&y);
        assert_eq!(
            p.signif,
            u256::new(
                0x6487ed5110b4611a62633145c06e0e68,
                0x948127044533e63a0105df531d899b4d,
            )
        );
        let a = BigFloat {
            sign: 1,
            exp: -255,
            signif: u256::new(
                0x7fffffffffffffffffffffffffffffff,
                0xfffffffffffffffffffff80000000000,
            ),
        };
        p.iadd(&a);
        assert_eq!(
            p.signif,
            u256::new(
                0x6487ed5110b4611a62633145c06e0e68,
                0x948127044533e63a0105df531d899b4d,
            )
        );
        x.imul_add(&y, &a);
        assert_eq!(
            x,
            BigFloat {
                sign: p.sign,
                exp: p.exp,
                signif: u256::new(
                    0x6487ed5110b4611a62633145c06e0e68,
                    0x948127044533e63a0105df531d899b4e,
                ),
            }
        );
    }
}
