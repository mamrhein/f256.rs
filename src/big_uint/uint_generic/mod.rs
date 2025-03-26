// ---------------------------------------------------------------------------
// Copyright:   (c) 2024 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

mod binops;

use alloc::vec::Vec;
use core::{fmt, ops::Rem};

use super::{BigUInt, DivRem, HiLo, U128};
use crate::big_uint::uint128::{u128_hi, u128_lo};

#[derive(Clone, Copy, Default, Eq, Ord, PartialOrd, PartialEq)]
pub(crate) struct UInt<SubUInt>
where
    SubUInt: BigUInt + HiLo,
{
    pub(crate) hi: SubUInt,
    pub(crate) lo: SubUInt,
}

impl<SubUInt> HiLo for UInt<SubUInt>
where
    SubUInt: BigUInt + HiLo,
{
    type T = SubUInt;

    // Number of u128 chunks in Self
    const N_CHUNKS: usize = 2 * SubUInt::N_CHUNKS;

    /// Returns a new instance of Self.
    #[inline(always)]
    fn from_hi_lo(hi: SubUInt, lo: SubUInt) -> Self {
        Self { hi, lo }
    }

    #[inline(always)]
    fn hi_t(&self) -> SubUInt {
        self.hi
    }

    #[inline(always)]
    fn lo_t(&self) -> SubUInt {
        self.lo
    }

    #[inline(always)]
    fn hi(&self) -> Self {
        Self::from_hi_lo(SubUInt::ZERO, self.hi)
    }

    #[inline(always)]
    fn lo(&self) -> Self {
        Self::from_hi_lo(SubUInt::ZERO, self.lo)
    }

    fn as_vec_u128(&self) -> Vec<u128> {
        [self.hi.as_vec_u128(), self.lo.as_vec_u128()].concat()
    }
}

impl<SubUInt> UInt<SubUInt>
where
    SubUInt: BigUInt + HiLo,
{
    // Specialized version adapted from
    // Henry S. Warren, Hacker’s Delight,
    // originally found at http://www.hackersdelight.org/HDcode/divlu.c.txt.
    // That code is in turn based on Algorithm D from
    // D. E. Knuth, The Art of Computer Programming, Vol. 2, Ch. 4.3.1,
    // adapted to the special case m = 4 and n = 2 and HI(x) < y (!).
    // The link given above does not exist anymore, but the code can still be
    // found at https://github.com/hcs0/Hackers-Delight/blob/master/divlu.c.txt.
    /// Returns `self` / rhs, `self` % rhs
    //noinspection DuplicatedCode
    pub(crate) fn div_rem_subuint_special(
        &self,
        rhs: &SubUInt,
    ) -> (Self, SubUInt) {
        debug_assert!(self.hi < *rhs);
        let n = SubUInt::BITS / 2;
        let two_pow_n = SubUInt::ONE << n;
        // Normalize dividend and divisor, so that the divisor has its highest
        // bit set, and get their n-bit parts.
        let shift = rhs.leading_zeros();
        let x = self << shift;
        let x32 = x.hi_t();
        let x1 = x.lo_t().hi();
        let x0 = x.lo_t().lo();
        let y = *rhs << shift;
        let y1 = y.hi();
        let y0 = y.lo();

        let (mut q1, mut rhat) = x32.div_rem(y1);
        // Now we have
        // q1 * y1 + rhat = x32
        // so that
        // q1 * y1 * 2ⁿ + rhat * 2ⁿ + x1 = x32 * 2ⁿ + x1
        while q1 >= two_pow_n || q1 * y0 > rhat * two_pow_n + x1 {
            q1.decr();
            rhat += &y1;
            if rhat >= two_pow_n {
                break;
            }
        }
        // The loop did not change the equation given above. It was terminated
        // if either q1 < 2ⁿ or rhat >= 2ⁿ or q1 * yn0 > rhat * 2ⁿ + x1.
        // In these cases follows:
        // q1 * y0 <= rhat * 2ⁿ + x1, therefor
        // q1 * y1 * 2ⁿ + q1 * y0 <= x32 * 2ⁿ + x1, and
        // q1 * y <= x32 * 2ⁿ + x1, and
        // x32 * 2ⁿ + x1 - q1 * y >= 0.
        // That means that the add-back step in Knuth's algorithm D is not
        // required.

        // Since the final quotient is < 2²⁵⁶, this must also be true for
        // x32 * 2ⁿ + x1 - q1 * y. Thus, in the following we can safely
        // ignore any possible overflow in x32 * 2ⁿ or q1 * y.
        let mut t = SubUInt::from_hi_lo(x32.lo_t(), x1.lo_t());
        t = t.wrapping_sub(&q1.wrapping_mul(&y));
        let (mut q0, mut rhat) = t.div_rem(y1);
        while q0 >= two_pow_n || q0 * y0 > rhat * two_pow_n + x0 {
            q0.decr();
            rhat += &y1;
            if rhat >= two_pow_n {
                break;
            }
        }
        // q = q1 * B + q0
        let q = (q1 << n) + q0;
        // Denormalize remainder
        let mut r = SubUInt::from_hi_lo(t.lo_t(), x0.lo_t());
        r = r.wrapping_sub(&q0.wrapping_mul(&y));
        r >>= shift;
        (Self::from_hi_lo(SubUInt::ZERO, q), r)
    }
}

impl<SubUInt> BigUInt for UInt<SubUInt>
where
    SubUInt: BigUInt + HiLo,
{
    const ZERO: Self = Self {
        hi: SubUInt::ZERO,
        lo: SubUInt::ZERO,
    };
    const ONE: Self = Self {
        hi: SubUInt::ZERO,
        lo: SubUInt::ONE,
    };
    const TWO: Self = Self {
        hi: SubUInt::ZERO,
        lo: SubUInt::TWO,
    };
    const MAX: Self = Self {
        hi: SubUInt::MAX,
        lo: SubUInt::MAX,
    };
    const TIE: Self = Self {
        hi: SubUInt::TIE,
        lo: SubUInt::ZERO,
    };

    /// Return true, if `self` is even.
    fn is_even(&self) -> bool {
        self.lo.is_even()
    }

    /// Return true, if `self` is odd.
    fn is_odd(&self) -> bool {
        self.lo.is_odd()
    }

    /// Returns the number of leading zeros in the binary representation of
    /// `self`.
    fn leading_zeros(&self) -> u32 {
        self.hi.leading_zeros()
            + self.hi.is_zero() as u32 * self.lo.leading_zeros()
    }

    /// Returns the number of trailing zeros in the binary representation of
    /// `self`.
    fn trailing_zeros(&self) -> u32 {
        self.lo.trailing_zeros()
            + self.lo.is_zero() as u32 * self.hi.trailing_zeros()
    }

    /// Add 1 to `self` inplace, wrapping around at Self::MAX.
    fn incr(&mut self) {
        self.lo = self.lo.wrapping_add(&SubUInt::ONE);
        self.hi.incr_if(self.lo.is_zero());
    }

    /// Subtract 1 from `self` inplace, wrapping around at Self::ZERO.
    #[inline]
    fn decr(&mut self) {
        self.hi.decr_if(self.lo.is_zero());
        self.lo = self.lo.wrapping_sub(&SubUInt::ONE);
    }

    /// Returns `self + rhs`, along with a boolean indicating whether an
    /// arithmetic overflow occurred.
    fn overflowing_add(&self, rhs: &Self) -> (Self, bool) {
        let (lo, carry) = self.lo.overflowing_add(&rhs.lo);
        let (hi, carry) = self.hi.carrying_add(&rhs.hi, carry);
        (Self { hi, lo }, carry)
    }

    /// Wrapping (modular) addition.
    /// Returns `self + rhs`, wrapping around at the boundary of the type.
    fn wrapping_add(&self, rhs: &Self) -> Self {
        let (lo, carry) = self.lo.overflowing_add(&rhs.lo);
        let mut hi = self.hi.wrapping_add(&rhs.hi);
        hi.incr_if(carry);
        Self { hi, lo }
    }

    /// Returns `self - rhs`, along with a boolean indicating whether an
    /// arithmetic overflow occurred.
    fn overflowing_sub(&self, rhs: &Self) -> (Self, bool) {
        let (lo, borrow) = self.lo.overflowing_sub(&rhs.lo);
        let (hi, borrow) = self.hi.borrowing_sub(rhs.hi, borrow);
        (Self { hi, lo }, borrow)
    }

    /// Wrapping (modular) subtraction.
    /// Returns `self - rhs`, wrapping around at the boundary of the type.
    fn wrapping_sub(&self, rhs: &Self) -> Self {
        let (lo, borrow) = self.lo.overflowing_sub(&rhs.lo);
        let mut hi = self.hi.wrapping_sub(&rhs.hi);
        hi.decr_if(borrow);
        Self { hi, lo }
    }

    /// Returns `self * rhs`, along with a boolean indicating whether an
    /// arithmetic overflow occurred.
    fn overflowing_mul(&self, rhs: &Self) -> (Self, bool) {
        let (lo, carry) = self.lo.widening_mul(&rhs.lo);
        let (hi, o1) = self.lo.carrying_mul(&rhs.hi, &carry);
        let (hi, o2) = self.hi.carrying_mul(&rhs.lo, &hi);
        let ovl = !(o1.is_zero()
            && o2.is_zero()
            && (self.hi.is_zero() || rhs.hi.is_zero()));
        (Self { hi, lo }, ovl)
    }

    fn widening_mul(&self, rhs: &Self) -> (Self, Self) {
        let (ll, carry) = self.lo.widening_mul(&rhs.lo);
        let (lh, hl) = self.lo.carrying_mul(&rhs.hi, &carry);
        let (lh, carry) = self.hi.carrying_mul(&rhs.lo, &lh);
        let (hl, ovfl) = hl.overflowing_add(&carry);
        let (hl, mut hh) = self.hi.carrying_mul(&rhs.hi, &hl);
        hh.incr_if(ovfl);
        let hi = Self { hi: hh, lo: hl };
        let lo = Self { hi: lh, lo: ll };
        (lo, hi)
    }

    /// Returns `self` / `rhs`, rounded tie to even.
    fn rounding_div(&self, rhs: &Self) -> Self {
        let (mut quot, rem) = self.div_rem(rhs);
        let tie = *rhs >> 1;
        if rem > tie || (rem == tie && quot.lo.is_odd()) {
            quot.incr();
        }
        quot
    }

    /// Returns `self` / `2ⁿ`, rounded tie to even.
    fn rounding_div_pow2(&self, n: u32) -> Self {
        let tie = Self {
            hi: SubUInt::ONE << (SubUInt::BITS - 1),
            lo: SubUInt::ZERO,
        };
        let (mut quot, rem) = self.widening_shr(n);
        if rem > tie || (rem == tie && quot.lo.is_odd()) {
            quot.incr();
        }
        quot
    }

    /// Returns `self` % `2ⁿ`, i.e. the n left-most bits of self.
    fn rem_pow2(&self, n: u32) -> Self {
        match n {
            0 => Self::ZERO,
            1.. if n < SubUInt::BITS => {
                let sh = SubUInt::BITS - n;
                Self::from_hi_lo(SubUInt::ZERO, (self.lo << sh) >> sh)
            }
            1.. if n == SubUInt::BITS => {
                Self::from_hi_lo(SubUInt::ZERO, self.lo)
            }
            1.. if n > SubUInt::BITS && n < Self::BITS => {
                let sh = Self::BITS - n;
                Self::from_hi_lo((self.hi << sh) >> sh, self.lo)
            }
            _ => *self,
        }
    }
}

impl<'a, SubUInt> From<&'a SubUInt> for UInt<SubUInt>
where
    SubUInt: BigUInt + HiLo,
{
    fn from(value: &'a SubUInt) -> Self {
        Self {
            hi: SubUInt::ZERO,
            lo: *value,
        }
    }
}

impl<SubUInt> From<u128> for UInt<SubUInt>
where
    SubUInt: BigUInt + HiLo,
{
    fn from(value: u128) -> Self {
        Self {
            hi: SubUInt::ZERO,
            lo: (&value).into(),
        }
    }
}

impl<'a, SubUInt> From<&'a u128> for UInt<SubUInt>
where
    SubUInt: BigUInt + HiLo,
{
    fn from(value: &'a u128) -> Self {
        Self {
            hi: SubUInt::ZERO,
            lo: value.into(),
        }
    }
}

impl<'a, SubUInt> From<&'a [u128]> for UInt<SubUInt>
where
    SubUInt: BigUInt + HiLo,
{
    fn from(value: &'a [u128]) -> Self {
        debug_assert!(value.len() <= (Self::BITS / 128) as usize);
        let idx = value.len().saturating_sub(SubUInt::N_CHUNKS);
        let (hi, lo) = value.split_at(idx);
        Self::from_hi_lo(SubUInt::from(hi), SubUInt::from(lo))
    }
}

impl<'a, SubUInt> From<&'a Vec<u128>> for UInt<SubUInt>
where
    SubUInt: BigUInt + HiLo,
{
    #[inline(always)]
    fn from(value: &'a Vec<u128>) -> Self {
        debug_assert!(value.len() == (Self::BITS / 128) as usize);
        Self::from_hi_lo(
            SubUInt::from(&value[0..value.len() / 2]),
            SubUInt::from(&value[value.len() / 2..]),
        )
    }
}

impl<SubUInt> fmt::Debug for UInt<SubUInt>
where
    SubUInt: BigUInt + HiLo,
{
    fn fmt(&self, form: &mut fmt::Formatter<'_>) -> fmt::Result {
        let chunks = self.as_vec_u128();
        write!(form, "(0x{:032x}", chunks[0])?;
        for c in chunks.iter().skip(1) {
            write!(form, ", 0x{:032x}", c)?;
        }
        write!(form, ")")
    }
}

impl<SubUInt> fmt::Display for UInt<SubUInt>
where
    SubUInt: BigUInt + HiLo,
{
    fn fmt(&self, form: &mut fmt::Formatter<'_>) -> fmt::Result {
        const SEGMENT_SIZE: usize = 38;
        const SEGMENT_BASE: u128 = 10_u128.pow(SEGMENT_SIZE as u32);
        if self.hi.is_zero() {
            return fmt::Display::fmt(&self.lo, form);
        }
        let mut segments = Vec::<u128>::new();
        let mut t = *self;
        let mut r: u128;
        while !t.is_zero() {
            (t, r) = t.div_rem(SEGMENT_BASE);
            segments.push(r);
        }
        let mut idx = segments.len() - 1;
        write!(form, "{}", segments[idx])?;
        while idx > 0 {
            idx -= 1;
            write!(form, "{:0SEGMENT_SIZE$}", segments[idx])?;
        }
        Ok(())
    }
}

pub type U256 = UInt<U128>;

impl U256 {
    /// Maximum number of decimal digits = ⌊log₁₀(2²⁵⁶ - 1)⌋.
    pub(crate) const MAX_N_DECIMAL_DIGITS: u32 = 77;

    /// Returns a new instance of Self.
    #[inline(always)]
    pub(crate) const fn new(hi: u128, lo: u128) -> Self {
        Self {
            hi: U128::new(hi),
            lo: U128::new(lo),
        }
    }

    /// Return true, if `self` == 0.
    #[inline(always)]
    pub(crate) const fn is_zero(&self) -> bool {
        self.hi.0 == 0_u128 && self.lo.0 == 0_u128
    }

    pub(crate) const fn shift_left(&self, shift: u32) -> Self {
        debug_assert!(shift < Self::BITS, "Shift with overflow");
        // TODO: change to exclusive range patterns when 1.82 got stable
        const MAX_SHIFT: u32 = U256::BITS - 1;
        const TIE: u32 = U256::BITS >> 1;
        const TIE_M1: u32 = TIE - 1;
        const TIE_P1: u32 = TIE + 1;
        match shift {
            1..=TIE_M1 => {
                let (lo, carry) =
                    (self.lo.0 << shift, self.lo.0 >> (u128::BITS - shift));
                let hi = (self.hi.0 << shift) | carry;
                Self::new(hi, lo)
            }
            TIE => Self::new(self.lo.0, 0_u128),
            TIE_P1..=MAX_SHIFT => {
                Self::new(self.lo.0 << (shift - TIE), 0_u128)
            }
            _ => *self,
        }
    }
}

impl DivRem<u64> for U256 {
    type Output = (Self, u64);

    #[inline(always)]
    fn div_rem(self, rhs: u64) -> Self::Output {
        (&self).div_rem(rhs)
    }
}

impl<'a> DivRem<u64> for &'a U256 {
    type Output = (U256, u64);

    fn div_rem(self, rhs: u64) -> Self::Output {
        let (quot_hi, rem) = self.hi.div_rem(rhs as u128);
        let mut t = U256::new(rem, self.lo.0);
        let (t, rem) = t.div_rem(rhs as u128);
        debug_assert!(t.hi.is_zero());
        (U256::from_hi_lo(quot_hi, t.lo), rem as u64)
    }
}

pub type U512 = UInt<UInt<U128>>;

impl U512 {
    /// Returns a new instance of Self.
    #[inline(always)]
    pub(crate) const fn new(hh: u128, hl: u128, lh: u128, ll: u128) -> Self {
        Self {
            hi: U256::new(hh, hl),
            lo: U256::new(lh, ll),
        }
    }
}

pub type U1024 = UInt<UInt<UInt<U128>>>;

#[cfg(test)]
mod widening_mul_tests {
    use super::*;

    #[test]
    fn test_u256_max_half() {
        let x = &U256::MAX >> 1;
        let z = (&x).widening_mul(&x);
        assert_eq!(z, (U256::ONE, &x >> 1));
    }

    #[test]
    fn test_u512_max_half() {
        let x = &U512::MAX >> 1;
        let z = (&x).widening_mul(&x);
        assert_eq!(z, (U512::ONE, &x >> 1));
    }
}

#[cfg(test)]
mod rounding_div_pow2_tests {
    use super::*;

    #[test]
    fn test_u256_rounding_div_pow2() {
        let u = U256::from_hi_lo(
            U128::from(0x00001000000000000000000000000003_u128),
            U128::from(0x00001000000000000000000000000002_u128),
        );
        let v = u.rounding_div_pow2(2);
        assert_eq!(v, &u >> 2);
        let v = u.rounding_div_pow2(17);
        assert_eq!(v, &u >> 17);
        let v = u.rounding_div_pow2(129);
        assert_eq!(v, &(&u >> 129) + &U256::ONE);
        let u = U256::from_hi_lo(
            U128::from(0x00001f6a7a2955385e583ebeff65cc22_u128),
            U128::from(0x6480ae685c3155a037f22051d5c9f93a_u128),
        );
        let mut v = u.clone();
        let n = 12;
        v = v.rounding_div_pow2(n);
        assert_eq!(v, &(&u >> 12) + &U256::ONE);
        let mut v = u.clone();
        let n = 137;
        v = v.rounding_div_pow2(n);
        assert_eq!(v, &u >> 137);
    }
}

#[cfg(test)]
mod rem_pow2_tests {
    use super::*;

    #[test]
    fn test_u256_rem_pow2() {
        let u = U256::from_hi_lo(
            U128::from(0x00381000000000000000000000000003_u128),
            U128::from(0x00007400000000000000000000000002_u128),
        );
        let v = u.rem_pow2(2);
        assert_eq!((v.hi, v.lo), (U128::ZERO, U128::from(2_u128)));
        let v = u.rem_pow2(108);
        assert_eq!(
            (v.hi, v.lo),
            (U128::ZERO, U128::from(0x400000000000000000000000002_u128))
        );
        let v = u.rem_pow2(128);
        assert_eq!((v.hi, v.lo), (U128::ZERO, v.lo));
        let v = u.rem_pow2(129);
        assert_eq!((v.hi, v.lo), (U128::from(1), v.lo));
        let v = u.rem_pow2(255);
        assert_eq!(v, u);
        let v = u.rem_pow2(256);
        assert_eq!(v, u);
        let v = u.rem_pow2(259);
        assert_eq!(v, u);
    }
}

#[cfg(test)]
mod from_int_tests {
    use super::*;

    #[test]
    fn test_from_subuint() {
        let x =
            U256::from_hi_lo(U128::new(u128::MAX >> 3), U128::new(7_u128));
        let y = U256::from(x);
        assert_eq!(y, x);
        let y = U512::from(&x);
        assert_eq!(y, U512::from_hi_lo(U256::ZERO, x));
        let z = U1024::from(&y);
        assert_eq!(z, U1024::from_hi_lo(U512::ZERO, y));
    }

    #[test]
    fn test_from_u128() {
        let x = (u128::MAX >> 3) - 7_u128;
        assert_eq!(
            U256::from(&x),
            U256::from_hi_lo(U128::ZERO, U128::from(&x))
        );
        assert_eq!(
            U512::from(&x),
            U512::from_hi_lo(U256::ZERO, U256::from(&x))
        );
        assert_eq!(
            U1024::from(&x),
            U1024::from_hi_lo(U512::ZERO, U512::from(&x))
        );
    }
}

#[cfg(test)]
mod from_slice_tests {
    use super::*;

    #[test]
    fn test_from_slice() {
        let a = [12_u128, 34_u128, 56_u128];
        let x = U256::from(&a[..1]);
        assert_eq!(x, U256::from_hi_lo(U128::ZERO, a[0].into()));
        let y = U256::from(&a[1..]);
        assert_eq!(y, U256::from_hi_lo(a[1].into(), a[2].into()));
        let z = U512::from(&a[..]);
        assert_eq!(z, U512::from_hi_lo(x, y));
        let z = U512::from(&a[..1]);
        assert_eq!(z, U512::from_hi_lo(U256::ZERO, a[0].into()));
        assert_eq!(U1024::from(&a[..0]), U1024::ZERO);
    }
}

#[cfg(test)]
mod into_vec_tests {
    use super::*;

    #[test]
    fn test_into_vec() {
        let a = [12_u128, 34_u128, 56_u128, 78_u128];
        let x = U512::from(&a[..]);
        let z = x.as_vec_u128();
        assert_eq!(z.len(), U512::N_CHUNKS);
        assert_eq!(z[..], a);
    }
}

#[cfg(test)]
mod u256_to_str_tests {
    use alloc::string::ToString;

    use super::*;

    #[test]
    fn test_zero() {
        let v = U256::ZERO;
        assert_eq!(v.to_string(), "0");
    }

    #[test]
    fn test_max() {
        let v = U256::MAX;
        assert_eq!(
            v.to_string(),
            "115792089237316195423570985008687907853269984665640564039457584007\
             913129639935"
        );
    }
}
