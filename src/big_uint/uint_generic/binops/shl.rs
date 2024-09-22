// ---------------------------------------------------------------------------
// Copyright:   (c) 2024 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use core::ops::{Shl, ShlAssign};

use super::{BigUInt, HiLo, UInt};

impl<SubUInt: BigUInt + HiLo> Shl<u32> for UInt<SubUInt> {
    type Output = Self;

    #[inline(always)]
    fn shl(self, shift: u32) -> Self::Output {
        &self << shift
    }
}

impl<SubUInt: BigUInt + HiLo> Shl<u32> for &UInt<SubUInt> {
    type Output = UInt<SubUInt>;

    fn shl(self, mut shift: u32) -> Self::Output {
        debug_assert!(
            shift < <UInt<SubUInt> as BigUInt>::BITS,
            "Shift with overflow"
        );
        if shift > 0 && shift < SubUInt::BITS {
            let (lo, carry) = self.lo.widening_shl(shift);
            let (hi, _) = self.hi.carrying_shl(shift, &carry);
            UInt::<SubUInt>::from_hi_lo(hi, lo)
        } else if shift == SubUInt::BITS {
            UInt::<SubUInt>::from_hi_lo(self.lo, SubUInt::ZERO)
        } else if shift > SubUInt::BITS {
            shift -= SubUInt::BITS;
            UInt::<SubUInt>::from_hi_lo(self.lo << shift, SubUInt::ZERO)
        } else {
            *self
        }
    }
}

impl<SubUInt: BigUInt + HiLo> ShlAssign<u32> for UInt<SubUInt> {
    fn shl_assign(&mut self, rhs: u32) {
        *self = &(*self) << rhs;
    }
}

#[cfg(test)]
mod u256_shl_tests {
    use super::*;
    use crate::{U128, U256};

    #[test]
    fn test_shl() {
        let u = U256 {
            hi: U128::MAX,
            lo: U128::MAX,
        };
        assert_eq!(&u << 0, u);
        let v = &u << 7;
        assert_eq!(
            v,
            U256 {
                hi: U128::MAX,
                lo: u.lo << 7,
            }
        );
        let v = &u << 128;
        assert_eq!(
            v,
            U256 {
                hi: U128::MAX,
                lo: U128::ZERO,
            }
        );
        let v = &u << 132;
        assert_eq!(
            v,
            U256 {
                hi: U128::MAX << 4,
                lo: U128::ZERO,
            }
        );
        let v = &u << 255;
        assert_eq!(
            v,
            U256 {
                hi: U128::ONE << 127,
                lo: U128::ZERO,
            }
        );
    }
}
