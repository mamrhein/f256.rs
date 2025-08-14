// ---------------------------------------------------------------------------
// Copyright:   (c) 2024 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use core::ops::{Add, AddAssign};

use super::{BigUInt, HiLo, UInt};

impl<SubUInt: BigUInt + HiLo> Add<u128> for UInt<SubUInt> {
    type Output = Self;

    #[inline(always)]
    fn add(self, rhs: u128) -> Self::Output {
        self + Self::from(&rhs)
    }
}

impl<SubUInt: BigUInt + HiLo> Add<SubUInt> for UInt<SubUInt> {
    type Output = Self;

    #[inline(always)]
    fn add(self, rhs: SubUInt) -> Self::Output {
        self + Self::from(&rhs)
    }
}

impl<SubUInt: BigUInt + HiLo> Add for UInt<SubUInt> {
    type Output = Self;

    #[inline(always)]
    fn add(self, rhs: Self) -> Self::Output {
        let (sum, carry) = self.overflowing_add(&rhs);
        assert!(!carry, "Attempt to add with overflow");
        sum
    }
}

impl<SubUInt: BigUInt + HiLo> Add for &UInt<SubUInt> {
    type Output = <UInt<SubUInt> as Add>::Output;

    fn add(self, rhs: Self) -> Self::Output {
        *self + *rhs
    }
}

impl<SubUInt: BigUInt + HiLo> AddAssign<&u128> for UInt<SubUInt> {
    fn add_assign(&mut self, rhs: &u128) {
        *self = *self + *rhs;
    }
}

impl<SubUInt: BigUInt + HiLo> AddAssign<&SubUInt> for UInt<SubUInt> {
    fn add_assign(&mut self, rhs: &SubUInt) {
        *self = *self + *rhs;
    }
}

impl<SubUInt: BigUInt + HiLo> AddAssign<&Self> for UInt<SubUInt> {
    fn add_assign(&mut self, rhs: &Self) {
        *self = &(*self) + rhs;
    }
}

#[cfg(test)]
mod u512_add_assign_tests {
    use super::*;
    use crate::{HiLo, U256, U512};

    #[test]
    fn test_add_assign_1() {
        let two = &U256::ONE + &U256::ONE;
        let mut v = U512::from_hi_lo(U256::ONE, U256::ONE);
        let w = v;
        let z = U512::from_hi_lo(two, two);
        v += &w;
        assert_eq!(v, z);
    }

    #[test]
    fn test_add_assign_2() {
        let mut v = U512::from_hi_lo(U256::ZERO, U256::MAX);
        let w = U512::from_hi_lo(U256::ONE, U256::ONE);
        let z = U512::from_hi_lo(&U256::ONE + &U256::ONE, U256::ZERO);
        v += &w;
        assert_eq!(v, z);
    }

    #[test]
    fn test_add_assign_3() {
        let mut v = U512::from_hi_lo(U256::ZERO, U256::MAX);
        let w = v;
        let z = U512::from_hi_lo(U256::ONE, &U256::MAX - &U256::ONE);
        v += &w;
        assert_eq!(v, z);
    }

    #[test]
    #[should_panic]
    fn test_add_assign_ovfl() {
        let mut v = U512::from_hi_lo(U256::ZERO, U256::MAX);
        let w = U512::from_hi_lo(U256::MAX, U256::ONE);
        v += &w;
    }
}
