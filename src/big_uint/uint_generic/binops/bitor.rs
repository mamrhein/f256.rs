// ---------------------------------------------------------------------------
// Copyright:   (c) 2024 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use core::ops::{BitOr, BitOrAssign};

use super::{BigUInt, HiLo, UInt};

impl<SubUInt: BigUInt + HiLo> BitOr for UInt<SubUInt> {
    type Output = Self;

    #[inline(always)]
    fn bitor(self, rhs: Self) -> Self::Output {
        &self | &rhs
    }
}

impl<'a, SubUInt: BigUInt + HiLo> BitOr for &'a UInt<SubUInt> {
    type Output = UInt<SubUInt>;

    #[inline(always)]
    fn bitor(self, rhs: Self) -> Self::Output {
        UInt::<SubUInt> {
            hi: self.hi | rhs.hi,
            lo: self.lo | rhs.lo,
        }
    }
}

impl<SubUInt: BigUInt + HiLo> BitOrAssign for UInt<SubUInt> {
    #[inline(always)]
    fn bitor_assign(&mut self, rhs: Self) {
        self.hi |= &rhs.hi;
        self.lo |= &rhs.lo;
    }
}

impl<'a, SubUInt: BigUInt + HiLo> BitOrAssign<&'a Self> for UInt<SubUInt> {
    fn bitor_assign(&mut self, rhs: &Self) {
        self.hi |= &rhs.hi;
        self.lo |= &rhs.lo;
    }
}