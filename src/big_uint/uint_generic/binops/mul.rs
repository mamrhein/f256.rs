// ---------------------------------------------------------------------------
// Copyright:   (c) 2024 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use core::ops::{Mul, MulAssign};

use super::{BigUInt, HiLo, UInt};

impl<SubUInt: BigUInt + HiLo> Mul<u128> for UInt<SubUInt> {
    type Output = Self;

    #[inline(always)]
    fn mul(self, rhs: u128) -> Self::Output {
        self * Self::from(&rhs)
    }
}

impl<SubUInt: BigUInt + HiLo> Mul<SubUInt> for UInt<SubUInt> {
    type Output = Self;

    #[inline(always)]
    fn mul(self, rhs: SubUInt) -> Self::Output {
        self * Self::from(&rhs)
    }
}

impl<SubUInt> Mul for UInt<SubUInt>
where
    SubUInt: BigUInt + HiLo,
{
    type Output = Self;

    #[inline(always)]
    fn mul(self, rhs: Self) -> Self::Output {
        (&self).mul(&rhs)
    }
}

impl<SubUInt> Mul for &UInt<SubUInt>
where
    SubUInt: BigUInt + HiLo,
{
    type Output = <UInt<SubUInt> as Mul>::Output;

    fn mul(self, rhs: Self) -> Self::Output {
        assert!(
            self.hi == <SubUInt as BigUInt>::ZERO
                || rhs.hi == <SubUInt as BigUInt>::ZERO,
            "Attempt to multiply with overflow."
        );
        let (lo, mut hi) = self.lo.widening_mul(&rhs.lo);
        let (mut t, mut ovfl) = self.lo.overflowing_mul(&rhs.hi);
        assert!(!ovfl, "Attempt to multiply with overflow.");
        (hi, ovfl) = hi.overflowing_add(&t);
        assert!(!ovfl, "Attempt to multiply with overflow.");
        (t, ovfl) = self.hi.overflowing_mul(&rhs.lo);
        assert!(!ovfl, "Attempt to multiply with overflow.");
        (hi, ovfl) = hi.overflowing_add(&t);
        assert!(!ovfl, "Attempt to multiply with overflow.");
        UInt::<SubUInt>::from_hi_lo(hi, lo)
    }
}

impl<SubUInt> MulAssign<&u128> for UInt<SubUInt>
where
    SubUInt: BigUInt + HiLo,
{
    fn mul_assign(&mut self, rhs: &u128) {
        *self = *self * *rhs;
    }
}

impl<SubUInt> MulAssign<&SubUInt> for UInt<SubUInt>
where
    SubUInt: BigUInt + HiLo,
{
    fn mul_assign(&mut self, rhs: &SubUInt) {
        *self = *self * *rhs;
    }
}

impl<SubUInt> MulAssign<&Self> for UInt<SubUInt>
where
    SubUInt: BigUInt + HiLo,
{
    fn mul_assign(&mut self, rhs: &Self) {
        *self = &(*self) * rhs;
    }
}
