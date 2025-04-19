// ---------------------------------------------------------------------------
// Copyright:   (c) 2024 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use core::ops::{Shr, ShrAssign};

use super::{BigUInt, HiLo, UInt};

impl<SubUInt: BigUInt + HiLo> Shr<u32> for UInt<SubUInt> {
    type Output = Self;

    #[inline(always)]
    fn shr(self, shift: u32) -> Self::Output {
        (&self).shr(shift)
    }
}

impl<SubUInt: BigUInt + HiLo> Shr<u32> for &UInt<SubUInt> {
    type Output = UInt<SubUInt>;

    #[inline(always)]
    fn shr(self, mut shift: u32) -> Self::Output {
        debug_assert!(
            shift < <UInt<SubUInt> as BigUInt>::BITS,
            "Shift with overflow"
        );
        if shift > 0 && shift < SubUInt::BITS {
            let (hi, carry) = self.hi.widening_shr(shift);
            let (lo, _) = self.lo.carrying_shr(shift, &carry);
            UInt::<SubUInt>::from_hi_lo(hi, lo)
        } else if shift == SubUInt::BITS {
            UInt::<SubUInt>::from_hi_lo(SubUInt::ZERO, self.hi)
        } else if shift > SubUInt::BITS {
            shift -= SubUInt::BITS;
            UInt::<SubUInt>::from_hi_lo(SubUInt::ZERO, self.hi >> shift)
        } else {
            *self
        }
    }
}

impl<SubUInt: BigUInt + HiLo> ShrAssign<u32> for UInt<SubUInt> {
    fn shr_assign(&mut self, rhs: u32) {
        *self = (*self) >> rhs;
    }
}

#[cfg(test)]
mod u256_shr_tests {
    use super::*;
    use crate::{U128, U256};

    #[test]
    fn test_shr() {
        let u = U256 {
            hi: U128::MAX,
            lo: U128::MAX,
        };
        assert_eq!(&u >> 0, u);
        let v = &u >> 3;
        assert_eq!(
            v,
            U256 {
                hi: u.hi >> 3,
                lo: U128::MAX,
            }
        );
        let v = &u >> 128;
        assert_eq!(
            v,
            U256 {
                hi: U128::ZERO,
                lo: U128::MAX,
            }
        );
        let v = &u >> 140;
        assert_eq!(
            v,
            U256 {
                hi: U128::ZERO,
                lo: U128::MAX >> 12,
            }
        );
    }
}
