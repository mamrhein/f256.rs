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
    ops::{AddAssign, Neg, SubAssign},
};

use crate::{
    big_uint::{u256, u512},
    EXP_BITS, f256,
};

use super::BigFloat;

/// Represents fixed-point numbers with 509 fractional bit in the range
/// [-4, 4 - 2⁻⁵⁰⁹].
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord)]
pub(super) struct FP509(u512);

impl FP509 {
    pub(super) const ZERO: Self = Self(u512::ZERO);
    pub(super) const ONE: Self =
        Self::new(1_u128 << 125, 0_u128, 0_u128, 0_u128);
    pub(super) const ONE_HALF: Self =
        Self::new(1_u128 << 124, 0_u128, 0_u128, 0_u128);
    pub(super) const EPSILON: Self =
        Self::new(0_u128, 0_u128, 0_u128, 1_u128);

    pub(super) const fn new(hh: u128, hl: u128, lh: u128, ll: u128) -> Self {
        Self(u512::new(u256::new(hh, hl), u256::new(lh, ll)))
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
    fn invert(&mut self) {
        self.0.lo.lo ^= u128::MAX;
        self.0.lo.hi ^= u128::MAX;
        self.0.hi.lo ^= u128::MAX;
        self.0.hi.hi ^= u128::MAX;
    }

    #[inline(always)]
    fn ineg(&mut self) {
        self.0.decr();
        self.invert();
    }

    #[inline(always)]
    fn iabs(&mut self) {
        if self.signum() < 0 {
            self.ineg();
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
        let mut carry = lo.hi.hi >> 125;
        let rem_hi = lo.hi.hi << 3;
        const TIE: u128 = 1_u128 << 127;
        carry += ((rem_hi > TIE)
            || (rem_hi == TIE && ((carry & 1_u128) == 1_u128)
                || lo.hi.lo != 0
                || !lo.lo.is_zero())) as u128;
        hi <<= 3;
        let mut ovl = false;
        (hi.lo, ovl) = hi.lo.overflowing_add(&u256::new(0_u128, carry));
        hi.hi += &u256::new(0_u128, ovl as u128);
        self.0 = hi;
        if signum < 0 {
            self.ineg();
        }
    }
}

impl From<&BigFloat> for FP509 {
    #[inline(always)]
    fn from(value: &BigFloat) -> Self {
        debug_assert!(value.exp <= 1);
        let mut res = Self(
            &u512::new(value.signif, u256::ZERO)
                >> value.exp.unsigned_abs() + 1,
        );
        if value.sign < 0 {
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
            // nlz < u256::BITS + EXP_BITS = 275 => shift right and round
            0..=274 => {
                let n = u256::BITS + EXP_BITS - nlz;
                fp_abs_signif.idiv_pow2(n);
                // shift left 1 bit in case rounding overflowed the hidden bit
                let sh = EXP_BITS - fp_abs_signif.lo.leading_zeros();
                fp_abs_signif.lo >>= sh;
                exp += sh as i32;
            }
            // nlz > u256::BITS + EXP_BITS = 275 => shift left
            276..=511 => fp_abs_signif <<= nlz - u256::BITS - EXP_BITS,
            _ => {}
        };
        Self::new(sign, exp, fp_abs_signif.lo)
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
        lhs.hi.hi ^= 1_u128 << 127;
        let mut rhs = other.0;
        rhs.hi.hi ^= 1_u128 << 127;
        lhs.partial_cmp(&rhs)
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
