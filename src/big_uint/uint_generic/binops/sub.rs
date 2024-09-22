// ---------------------------------------------------------------------------
// Copyright:   (c) 2024 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use core::ops::{Sub, SubAssign};

use super::{BigUInt, HiLo, UInt};

impl<SubUInt: BigUInt + HiLo> Sub<u128> for UInt<SubUInt> {
    type Output = Self;

    #[inline(always)]
    fn sub(self, rhs: u128) -> Self::Output {
        &self - &Self::from(&rhs)
    }
}

impl<SubUInt: BigUInt + HiLo> Sub<SubUInt> for UInt<SubUInt> {
    type Output = Self;

    #[inline(always)]
    fn sub(self, rhs: SubUInt) -> Self::Output {
        &self - &Self::from(&rhs)
    }
}

impl<SubUInt> Sub for UInt<SubUInt>
where
    SubUInt: BigUInt + HiLo,
{
    type Output = Self;

    #[inline(always)]
    fn sub(self, rhs: Self) -> Self::Output {
        &self - &rhs
    }
}

impl<SubUInt> Sub for &UInt<SubUInt>
where
    SubUInt: BigUInt + HiLo,
{
    type Output = <UInt<SubUInt> as Sub>::Output;

    fn sub(self, rhs: Self) -> Self::Output {
        let (diff, borrow) = self.overflowing_sub(rhs);
        assert!(!borrow, "Attempt to subtract with overflow");
        diff
    }
}

impl<SubUInt> SubAssign<&u128> for UInt<SubUInt>
where
    SubUInt: BigUInt + HiLo,
{
    fn sub_assign(&mut self, rhs: &u128) {
        *self = *self - *rhs;
    }
}

impl<SubUInt> SubAssign<&SubUInt> for UInt<SubUInt>
where
    SubUInt: BigUInt + HiLo,
{
    fn sub_assign(&mut self, rhs: &SubUInt) {
        *self = *self - *rhs;
    }
}

impl<SubUInt> SubAssign<&Self> for UInt<SubUInt>
where
    SubUInt: BigUInt + HiLo,
{
    fn sub_assign(&mut self, rhs: &Self) {
        *self = &(*self) - rhs;
    }
}

#[cfg(test)]
mod u512_sub_assign_tests {
    use super::*;
    use crate::{U256, U512};

    #[test]
    fn test_sub_assign_1() {
        let two = &U256::ONE + &U256::ONE;
        let mut v = U512::from_hi_lo(two, two);
        let w = U512::from_hi_lo(U256::ONE, U256::ONE);
        v -= &w;
        assert_eq!(v, w);
    }

    #[test]
    fn test_sub_assign_2() {
        let mut v = U512::from_hi_lo(U256::MAX, U256::ZERO);
        let w = U512::from_hi_lo(U256::ZERO, U256::ONE);
        let z = U512::from_hi_lo(&U256::MAX - &U256::ONE, U256::MAX);
        v -= &w;
        assert_eq!(v, z);
    }

    #[test]
    fn test_sub_assign_3() {
        let mut v = U512::from_hi_lo(U256::ONE, U256::MAX);
        let w = U512::from_hi_lo(U256::ONE, &U256::MAX - &U256::ONE);
        let z = U512::from_hi_lo(U256::ZERO, U256::ONE);
        v -= &w;
        assert_eq!(v, z);
    }

    #[test]
    #[should_panic]
    fn test_sub_assign_ovfl() {
        let mut v = U512::from_hi_lo(U256::ONE, &U256::MAX - &U256::ONE);
        let w = U512::from_hi_lo(U256::ONE, U256::MAX);
        v -= &w;
    }
}
