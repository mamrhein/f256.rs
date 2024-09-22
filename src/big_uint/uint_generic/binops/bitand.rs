// ---------------------------------------------------------------------------
// Copyright:   (c) 2024 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use core::ops::{BitAnd, BitAndAssign};

use super::{BigUInt, UInt};
use crate::HiLo;

impl<SubUInt: BigUInt + HiLo> BitAnd for UInt<SubUInt> {
    type Output = Self;

    #[inline(always)]
    fn bitand(self, rhs: Self) -> Self::Output {
        &self & &rhs
    }
}

impl<SubUInt: BigUInt + HiLo> BitAnd for &UInt<SubUInt> {
    type Output = UInt<SubUInt>;

    #[inline(always)]
    fn bitand(self, rhs: Self) -> Self::Output {
        UInt::<SubUInt> {
            hi: self.hi & rhs.hi,
            lo: self.lo & rhs.lo,
        }
    }
}

impl<'a, SubUInt: BigUInt + HiLo> BitAndAssign<&'a Self> for UInt<SubUInt> {
    fn bitand_assign(&mut self, rhs: &Self) {
        self.hi &= &rhs.hi;
        self.lo &= &rhs.lo;
    }
}
