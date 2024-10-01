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

const FRAC_1_OVER_256: FP509 = FP509::new(
    0x00200000000000000000000000000000,
    0x00000000000000000000000000000000,
    0x00000000000000000000000000000000,
    0x00000000000000000000000000000000,
);
const FRAC_1_OVER_256_HI_TZ: u32 = 117;

/// Represents fixed-point numbers with 509 fractional bits in the range
/// [-4, 4 - 2⁻⁵⁰⁹].
#[derive(Clone, Copy, Default, Eq, PartialEq)]
pub(super) struct FP509(U512);

impl FP509 {
    pub(super) const FRACTION_BITS: u32 = 509;
    pub(super) const ZERO: Self = Self(U512::ZERO);
    pub(super) const ONE: Self =
        Self::new(1_u128 << 125, 0_u128, 0_u128, 0_u128);
    pub(super) const TWO: Self =
        Self::new(1_u128 << 126, 0_u128, 0_u128, 0_u128);
    pub(super) const ONE_HALF: Self =
        Self::new(1_u128 << 124, 0_u128, 0_u128, 0_u128);
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
            assert!(res < FP509::TWO);
            res.0 <<= 1;
            res.ineg();
            res
        } else {
            let mut res = *self;
            assert!(res < FP509::TWO);
            res.0 <<= 1;
            res
        }
    }

    /// self =  ◯(self * rhs)
    pub(super) fn imul_round(&mut self, rhs: &Self) {
        let signum = self.signum() * rhs.signum();
        self.iabs();
        let mut rhs = *rhs;
        rhs.iabs();
        let (lo, mut hi) = self.0.widening_mul(&rhs.0);
        // To get an FP509 in hi, shift (hi, lo) left by 3 bits and round
        let mut carry = lo.hi.hi.0 >> 125;
        let rem_hi = lo.hi.hi.0 << 3;
        const TIE: u128 = 1_u128 << 127;
        carry += ((rem_hi > TIE)
            || (rem_hi == TIE && ((carry & 1_u128) == 1_u128)
                || lo.hi.lo.0 != 0
                || !lo.lo.is_zero())) as u128;
        hi <<= 3;
        let mut ovl = false;
        (hi.lo, ovl) = hi.lo.overflowing_add(&U256::new(0_u128, carry));
        hi.hi.incr_if(ovl);
        self.0 = hi;
        if signum < 0 {
            self.ineg();
        }
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
        let mut c = FP509::new(q << FRAC_1_OVER_256_HI_TZ, 0, 0, 0);
        self -= &c;
        (q as i32, c, self)
    }
}

impl From<&BigFloat> for FP509 {
    #[inline(always)]
    fn from(value: &BigFloat) -> Self {
        debug_assert!(value.exp <= 1);
        let sh = value.exp.unsigned_abs() + 1;
        if sh >= U512::BITS {
            return FP509::ZERO;
        }
        let mut res = Self(&U512::from_hi_lo(value.signif, U256::ZERO) >> sh);
        if value.signum < 0 {
            res.ineg();
        }
        res
    }
}

impl From<&FP509> for BigFloat {
    fn from(value: &FP509) -> Self {
        let signum = value.signum();
        let mut fp_abs_signif = if signum >= 0 {
            value.0
        } else {
            let mut t = *value;
            t.ineg();
            t.0
        };
        let nlz = fp_abs_signif.leading_zeros();
        let mut exp = 2 - nlz as i32;
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
            // nlz > N = 257 => shift left
            258..=511 => fp_abs_signif <<= nlz - N,
            _ => {}
        };
        Self::new(signum, exp, (fp_abs_signif.lo.hi.0, fp_abs_signif.lo.lo.0))
    }
}

impl From<&f256> for FP509 {
    fn from(value: &f256) -> Self {
        const FOUR: f256 = f256::from_u64(4);
        const RADIX_ADJ: u32 =
            FP509::FRACTION_BITS - U256::BITS - FRACTION_BITS;
        debug_assert!(value < &FOUR);
        let (sign, mut exp, signif) = split_f256_enc(value);
        // Compensate fraction bias
        exp += FRACTION_BITS as i32;
        let shl = RADIX_ADJ + exp.clamp(0, 2) as u32;
        let shr = exp.clamp(-510, 0).unsigned_abs();
        let mut res =
            Self(&U512::from_hi_lo(&signif << shl, U256::ZERO) >> shr);
        if value.is_sign_negative() {
            res.ineg();
        }
        res
    }
}

impl From<&FP509> for f256 {
    fn from(value: &FP509) -> Self {
        let sign = value.signum() as u32 >> 31;
        let mut fp_abs_signif = if sign == 0 {
            value.0
        } else {
            let mut t = *value;
            t.ineg();
            t.0
        };
        let nlz = fp_abs_signif.leading_zeros();
        let mut exp = 2 - nlz as i32;
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
            // nlz > U256::BITS + EXP_BITS = 275 => shift left
            276..=511 => fp_abs_signif <<= nlz - U256::BITS - EXP_BITS,
            _ => {
                return f256::ZERO;
            }
        };
        Self::new(sign, exp, fp_abs_signif.lo)
    }
}

impl From<U512> for FP509 {
    #[inline(always)]
    fn from(value: U512) -> Self {
        Self(value)
    }
}

impl Neg for FP509 {
    type Output = Self;

    fn neg(mut self) -> Self::Output {
        self.ineg();
        self
    }
}

impl PartialOrd for FP509 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let mut lhs = self.0;
        lhs.hi.hi.0 ^= 1_u128 << 127;
        let mut rhs = other.0;
        rhs.hi.hi.0 ^= 1_u128 << 127;
        lhs.partial_cmp(&rhs)
    }
}

impl Ord for FP509 {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl AddAssign<&FP509> for FP509 {
    #[inline(always)]
    fn add_assign(&mut self, rhs: &FP509) {
        let mut carry = false;
        (self.0.lo, carry) = self.0.lo.overflowing_add(&rhs.0.lo);
        (self.0.hi, carry) = self.0.hi.carrying_add(&rhs.0.hi, carry);
    }
}

impl SubAssign<&FP509> for FP509 {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: &FP509) {
        *self += &rhs.neg();
    }
}

impl Sub for &FP509 {
    type Output = FP509;

    fn sub(self, rhs: Self) -> Self::Output {
        let mut res = *self;
        res -= rhs;
        res
    }
}

impl MulAssign<&Self> for FP509 {
    #[inline(always)]
    fn mul_assign(&mut self, rhs: &Self) {
        self.imul_round(rhs);
    }
}

impl DivAssign<&Self> for FP509 {
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

impl fmt::Debug for FP509 {
    fn fmt(&self, form: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut f = *self;
        f.iabs();
        fmt::Debug::fmt(&(self.signum(), f.0), form)
    }
}

#[cfg(test)]
mod test_fp509 {
    use super::*;

    #[test]
    fn test_imul_round() {
        let mut a = FP509::new(
            0x00000000000000000000000000000000,
            0x000067b2347253a16bd31e2c570f6274,
            0xbcff2caa49cd10c1720c6cd864f126a4,
            0x41d4e7c8a606052efcd32ed46b74b006,
        );
        let b = FP509::new(
            0x00888888888888888888888888888888,
            0x88888888888888888888888888888888,
            0x88888888888888888888888888888888,
            0x88888888888888888888888888888889,
        );
        let c = FP509::new(
            0x00000000000000000000000000000000,
            0x000001ba6fceb49806105191ce4041a4,
            0x143773f1c57f26ade408bd8c23d0d1b5,
            0xf007d0a9be8019afb769eb727941f1de,
        );
        a.imul_round(&b);
        assert_eq!(a, c);
    }

    #[test]
    fn test_neg_one() {
        let x = FP509::ONE;
        let y = -x;
        let mut z = x;
        z.imul_round(&y);
        assert_eq!(y, z);
    }

    #[test]
    fn test_neg_neg() {
        let x = FP509::new(
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
        let x = FP509::ONE;
        let y = -x;
        assert_eq!(f256::from(&x), f256::ONE);
        assert_eq!(f256::from(&y), f256::NEG_ONE);
        assert_eq!(BigFloat::from(&x), BigFloat::ONE);
        assert_eq!(BigFloat::from(&y), BigFloat::NEG_ONE);
    }
}
