// ---------------------------------------------------------------------------
// Copyright:   (c) 2024 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use core::{
    cmp::Ordering,
    fmt,
    ops::{AddAssign, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};

use super::BigFloat;
use crate::{
    f256, split_f256_enc, BigUInt, HiLo, EXP_BITS, FRACTION_BITS,
    SIGNIFICAND_BITS, U1024, U256, U512,
};

const HI_FRACTION_BITS: u32 = FP492::FRACTION_BITS - 3 * 128;
const FRAC_1_OVER_256_HI_TZ: u32 = FP492::FRACTION_BITS - 3 * 128 - 8;
const FRAC_1_OVER_256: FP492 =
    FP492::new(1_u128 << FRAC_1_OVER_256_HI_TZ, 0_u128, 0_u128, 0_u128);

/// Represents fixed-point numbers with 492 fractional bits in the range
/// [-2¹⁹, 2¹⁹ - 2⁻⁴⁹²].
#[derive(Clone, Copy, Default, Eq, PartialEq)]
pub(super) struct FP492(U512);

impl FP492 {
    pub(super) const FRACTION_BITS: u32 = 492;
    pub(super) const INT_BITS: u32 = U512::BITS - Self::FRACTION_BITS - 1;
    pub(super) const ZERO: Self = Self(U512::ZERO);
    pub(super) const ONE: Self =
        Self::new(1_u128 << HI_FRACTION_BITS, 0_u128, 0_u128, 0_u128);
    pub(super) const TWO: Self =
        Self::new(1_u128 << HI_FRACTION_BITS + 1, 0_u128, 0_u128, 0_u128);
    pub(super) const ONE_HALF: Self =
        Self::new(1_u128 << HI_FRACTION_BITS - 1, 0_u128, 0_u128, 0_u128);
    pub(super) const EPSILON: Self =
        Self::new(0_u128, 0_u128, 0_u128, 1_u128);

    #[inline(always)]
    pub(super) const fn new(hh: u128, hl: u128, lh: u128, ll: u128) -> Self {
        Self(U512::new(hh, hl, lh, ll))
    }

    #[inline(always)]
    fn leading_zeroes(&self) -> u32 {
        self.0.leading_zeros()
    }

    #[inline(always)]
    pub(super) fn is_zero(&self) -> bool {
        self.0.is_zero()
    }

    #[inline(always)]
    fn signum(&self) -> i32 {
        match self.0.leading_zeros() {
            0 => -1,
            512 => 0,
            _ => 1,
        }
    }

    #[inline(always)]
    pub(super) fn is_sign_positive(&self) -> bool {
        self.0.hi.hi.leading_zeros() != 0
    }

    #[inline(always)]
    pub(super) fn is_sign_negative(&self) -> bool {
        self.0.hi.hi.leading_zeros() == 0
    }

    #[inline(always)]
    fn invert(&mut self) {
        self.0.lo.lo.0 ^= u128::MAX;
        self.0.lo.hi.0 ^= u128::MAX;
        self.0.hi.lo.0 ^= u128::MAX;
        self.0.hi.hi.0 ^= u128::MAX;
    }

    #[inline(always)]
    fn ineg(&mut self) {
        self.0.decr();
        self.invert();
    }

    #[inline(always)]
    pub(crate) fn iabs(&mut self) {
        if self.is_sign_negative() {
            self.ineg();
        }
    }

    #[inline(always)]
    pub(crate) fn abs(mut self) -> Self {
        if self.is_sign_negative() {
            self.ineg();
        }
        self
    }

    pub(super) fn mul2(&self) -> Self {
        if self.signum() < 0 {
            let mut res = self.abs();
            assert!(res.0.hi.hi.0.leading_zeros() >= 2);
            res.0 <<= 1;
            res.ineg();
            res
        } else {
            let mut res = *self;
            assert!(res.0.hi.hi.0.leading_zeros() >= 2);
            res.0 <<= 1;
            res
        }
    }

    /// self =  ◯₄₉₂(self * rhs)
    pub(super) fn imul_round(&mut self, rhs: &Self) {
        let signum = self.signum() * rhs.signum();
        self.iabs();
        let mut rhs = *rhs;
        rhs.iabs();
        let (lo, mut hi) = self.0.widening_mul(&rhs.0);
        // To get an FP492 in hi, shift (hi, lo) left by 21 bits and round
        const SHL: u32 = U512::BITS - FP492::FRACTION_BITS;
        let mut carry = lo.hi.hi.0 >> (u128::BITS - SHL);
        let rem_hi = lo.hi.hi.0 << SHL;
        const TIE: u128 = 1_u128 << 127;
        carry += ((rem_hi > TIE)
            || (rem_hi == TIE && ((carry & 1_u128) == 1_u128)
                || lo.hi.lo.0 != 0
                || !lo.lo.is_zero())) as u128;
        hi <<= SHL;
        let mut ovl = false;
        (hi.lo, ovl) = hi.lo.overflowing_add(&U256::new(0_u128, carry));
        hi.hi.incr_if(ovl);
        self.0 = hi;
        if signum < 0 {
            self.ineg();
        }
    }

    /// Returns self + ⌊self * 2⁻ⁿ⌋₄₉₂
    #[inline(always)]
    pub(super) fn add_self_shr(self, n: u32) -> Self {
        Self(self.0 + (self.0 >> n))
    }

    /// Returns ⌊self * 256⌋, ⌊self * 256⌋ / 256, self - ⌊self * 256⌋ / 256
    pub(super) fn divmod_1_over_256(mut self) -> (i32, Self, Self) {
        let signum = self.signum();
        if signum == 0 {
            return (0, self, self);
        }
        if signum == -1 {
            self.iabs();
        }
        let q = self.0.hi.hi.0 >> FRAC_1_OVER_256_HI_TZ;
        let mut c = FP492::new(q << FRAC_1_OVER_256_HI_TZ, 0, 0, 0);
        self -= &c;
        (q as i32, c, self)
    }
}

impl From<i32> for FP492 {
    fn from(i: i32) -> Self {
        debug_assert!(i.unsigned_abs() < 1_u32 << Self::INT_BITS);
        let mut t = Self::new(
            (i.unsigned_abs() as u128) << HI_FRACTION_BITS,
            0_u128,
            0_u128,
            0_u128,
        );
        if i.is_negative() {
            t.ineg();
        }
        t
    }
}

impl From<&BigFloat> for FP492 {
    fn from(value: &BigFloat) -> Self {
        debug_assert!(value.exp <= Self::INT_BITS as i32);
        let sh = (Self::INT_BITS as i32 - 1 - value.exp) as u32;
        if sh >= U512::BITS {
            return FP492::ZERO;
        }
        let mut res = Self(&U512::from_hi_lo(value.signif, U256::ZERO) >> sh);
        if value.signum < 0 {
            res.ineg();
        }
        res
    }
}

impl From<&FP492> for BigFloat {
    fn from(value: &FP492) -> Self {
        let signum = value.signum();
        let mut fp_abs_signif = if signum >= 0 {
            value.0
        } else {
            let mut t = *value;
            t.ineg();
            t.0
        };
        let nlz = fp_abs_signif.leading_zeros();
        let mut exp = FP492::INT_BITS as i32 - nlz as i32;
        const N: u32 = U512::BITS - BigFloat::FRACTION_BITS - 1;
        match nlz {
            // nlz < N = 257 => shift right and round
            0..=256 => {
                let n = N - nlz;
                fp_abs_signif = fp_abs_signif.rounding_div_pow2(n);
                // shift left 1 bit in case rounding overflowed to the
                // reserved bit
                let sh = 1 - fp_abs_signif.lo.leading_zeros();
                fp_abs_signif.lo >>= sh;
                exp += sh as i32;
            }
            257 => {}
            // nlz > N = 257 => shift left
            258..=511 => fp_abs_signif <<= nlz - N,
            _ => {
                return BigFloat::ZERO;
            }
        };
        Self::new(signum, exp, (fp_abs_signif.lo.hi.0, fp_abs_signif.lo.lo.0))
    }
}

impl From<&f256> for FP492 {
    fn from(value: &f256) -> Self {
        const LIM: f256 = f256::from_u64(1 << FP492::INT_BITS);
        debug_assert!(value < &LIM);
        let (sign, mut exp, signif) = split_f256_enc(value);
        // Compensate fraction bias
        exp += FRACTION_BITS as i32;
        let shl = exp.clamp(0, Self::INT_BITS as i32) as u32;
        let shr = exp
            .clamp(Self::INT_BITS as i32 + 1 - U512::BITS as i32, 0)
            .unsigned_abs();
        let mut res =
            Self(&U512::from_hi_lo(&signif << shl, U256::ZERO) >> shr);
        if value.is_sign_negative() {
            res.ineg();
        }
        res
    }
}

impl From<&FP492> for f256 {
    fn from(value: &FP492) -> Self {
        let sign = value.signum() as u32 >> 31;
        let mut fp_abs_signif = if sign == 0 {
            value.0
        } else {
            let mut t = *value;
            t.ineg();
            t.0
        };
        let nlz = fp_abs_signif.leading_zeros();
        let mut exp = FP492::INT_BITS as i32 - nlz as i32;
        match nlz {
            // nlz < U256::BITS + EXP_BITS = 275 => shift right and round
            0..=274 => {
                let n = U256::BITS + EXP_BITS - nlz;
                fp_abs_signif = fp_abs_signif.rounding_div_pow2(n);
                // shift left 1 bit in case rounding overflowed the hidden bit
                let sh = EXP_BITS - fp_abs_signif.lo.leading_zeros();
                fp_abs_signif.lo >>= sh;
                exp += sh as i32;
            }
            275 => {}
            // nlz > U256::BITS + EXP_BITS = 275 => shift left
            276..=511 => fp_abs_signif <<= nlz - U256::BITS - EXP_BITS,
            _ => {
                return f256::ZERO;
            }
        };
        Self::new(sign, exp, fp_abs_signif.lo)
    }
}

impl From<U512> for FP492 {
    #[inline(always)]
    fn from(value: U512) -> Self {
        Self(value)
    }
}

impl Neg for FP492 {
    type Output = Self;

    #[inline(always)]
    fn neg(mut self) -> Self::Output {
        self.ineg();
        self
    }
}

impl PartialOrd for FP492 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let mut lhs = self.0;
        lhs.hi.hi.0 ^= 1_u128 << 127;
        let mut rhs = other.0;
        rhs.hi.hi.0 ^= 1_u128 << 127;
        lhs.partial_cmp(&rhs)
    }
}

impl Ord for FP492 {
    #[inline(always)]
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl AddAssign<&FP492> for FP492 {
    #[inline(always)]
    fn add_assign(&mut self, rhs: &FP492) {
        let mut carry = false;
        (self.0.lo, carry) = self.0.lo.overflowing_add(&rhs.0.lo);
        (self.0.hi, carry) = self.0.hi.carrying_add(&rhs.0.hi, carry);
    }
}

impl SubAssign<&FP492> for FP492 {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: &FP492) {
        *self += &rhs.neg();
    }
}

impl Sub for &FP492 {
    type Output = FP492;

    #[inline(always)]
    fn sub(self, rhs: Self) -> Self::Output {
        let mut res = *self;
        res -= rhs;
        res
    }
}

impl MulAssign<&Self> for FP492 {
    #[inline(always)]
    fn mul_assign(&mut self, rhs: &Self) {
        self.imul_round(rhs);
    }
}

impl DivAssign<&Self> for FP492 {
    fn div_assign(&mut self, rhs: &Self) {
        debug_assert!(self.is_sign_positive());
        debug_assert!(rhs.is_sign_positive());
        debug_assert!(
            self.0.hi.hi.leading_zeros() > rhs.0.hi.hi.leading_zeros()
        );
        let mut x =
            U1024::from_hi_lo(U512::ZERO, self.0) << Self::FRACTION_BITS;
        let y = rhs.0;
        let tie = y >> 1;
        let (mut quot, rem) = x.div_rem_subuint_special(&y);
        quot.incr_if(rem > tie || rem == tie && quot.is_odd());
        debug_assert!(quot.hi.is_zero());
        self.0 = quot.lo;
    }
}

impl fmt::Debug for FP492 {
    fn fmt(&self, form: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut f = *self;
        f.iabs();
        fmt::Debug::fmt(&(self.signum(), f.0), form)
    }
}

#[cfg(test)]
mod test_fp492 {
    use super::*;

    #[test]
    fn test_iadd() {
        let x = FP492::from(&f256::from(7.24001));
        let y = FP492::from(-3);
        let mut z = x;
        z += &y;
        assert_eq!(z, FP492::from(&f256::from(4.24001)));
        let mut z = y;
        z += &(-x);
        assert_eq!(z, FP492::from(&f256::from(-10.24001)));
        let mut a = -y;
        a += &x;
        assert_eq!(z, -a);
    }

    #[test]
    fn test_imul_round() {
        let mut a = FP492::new(
            0x00000000000000000000000000000000,
            0x000067b2347253a16bd31e2c570f6274,
            0xbcff2caa49cd10c1720c6cd864f126a4,
            0x41d4e7c8a606052efcd32ed46b74b006,
        );
        let b = FP492::new(
            0x00888888888888888888888888888888,
            0x88888888888888888888888888888888,
            0x88888888888888888888888888888888,
            0x88888888888888888888888888888889,
        );
        let c = FP492::new(
            0x00000000000000000000000000000000,
            0x0374df9d69300c20a3239c808348286e,
            0xe7e38afe4d5bc8117b1847a1a36be00f,
            0xa1537d00335f6ed3d6e4f283e3bbeeef,
        );
        a.imul_round(&b);
        assert_eq!(a, c);
    }

    #[test]
    fn test_neg_one() {
        let x = FP492::ONE;
        let y = -x;
        let mut z = x;
        z.imul_round(&y);
        assert_eq!(y, z);
    }

    #[test]
    fn test_neg_neg() {
        let x = FP492::new(
            0x00088888888888888888888888888888,
            0x000000dd37e75a4c030828c8e72020d2,
            0x0a1bb9f8e2bf9356f2045ec611e868da,
            0xf803e854df400cd7dbb4f5b93ca0f8ef,
        );
        let mut y = x;
        y.ineg();
        y.ineg();
        assert_eq!(x, y);
    }
}

#[cfg(test)]
mod test_fp509_conv {
    use super::*;

    #[test]
    fn test_one() {
        let x = FP492::ONE;
        let y = -x;
        assert_eq!(f256::from(&x), f256::ONE);
        assert_eq!(f256::from(&y), f256::NEG_ONE);
        assert_eq!(BigFloat::from(&x), BigFloat::ONE);
        assert_eq!(BigFloat::from(&y), BigFloat::NEG_ONE);
    }

    #[test]
    fn test_some() {
        let x = f256::from(7) + f256::from(f64::EPSILON) * f256::TEN;
        let y = BigFloat::from(&x);
        let a = FP492::from(&x);
        let b = FP492::from(&y);
        assert_eq!(f256::from(&a), x);
        assert_eq!(BigFloat::from(&b), y);
        assert_eq!(a, b);
    }

    #[test]
    fn test_from_i32() {
        let x = FP492::from(7);
        let y = FP492::from(&f256::from(7));
        assert_eq!(x, y);
        let x = FP492::from(-7);
        let y = FP492::from(&f256::from(-7));
        assert_eq!(x, y);
    }
}
