// ---------------------------------------------------------------------------
// Copyright:   (c) 2022 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use core::{
    cmp::Ordering,
    mem::size_of,
    ops::{AddAssign, MulAssign, Shl, ShlAssign, Shr, ShrAssign, SubAssign},
};

#[inline(always)]
const fn u128_hi(u: u128) -> u128 {
    u >> 64
}

#[inline(always)]
const fn u128_lo(u: u128) -> u128 {
    u & 0xffffffffffffffff
}

// Calculate z = x * y.
pub(crate) const fn u128_widening_mul(x: u128, y: u128) -> u256 {
    let xh = u128_hi(x);
    let xl = u128_lo(x);
    let yh = u128_hi(y);
    let yl = u128_lo(y);
    let mut t = xl * yl;
    let mut rl = u128_lo(t);
    t = xl * yh + u128_hi(t);
    let mut rh = u128_hi(t);
    t = xh * yl + u128_lo(t);
    rl += u128_lo(t) << 64;
    rh += xh * yh + u128_hi(t);
    u256::new(rh, rl)
}

/// Helper type representing unsigned integers of 256 bits.
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialOrd, PartialEq)]
pub(crate) struct u256 {
    pub(crate) hi: u128,
    pub(crate) lo: u128,
}

impl u256 {
    /// The size of this integer type in bits.
    #[allow(clippy::cast_possible_truncation)]
    pub(crate) const BITS: u32 = size_of::<Self>() as u32 * 8;

    /// Additive identity = 0.
    pub(crate) const ZERO: Self = Self::new(0, 0);

    /// Maximum value = 2²⁵⁶ - 1.
    pub(crate) const MAX: Self = Self::new(u128::MAX, u128::MAX);

    /// Maximum number of decimal digits = ⌊log₁₀(2²⁵⁶ - 1)⌋.
    pub(crate) const MAX_N_DECIMAL_DIGITS: u32 = 77;

    /// Create an `u256` value from two u128 values.
    #[inline(always)]
    pub(crate) const fn new(hi: u128, lo: u128) -> Self {
        Self { hi, lo }
    }

    /// Return true, if `self` == 0.
    #[inline]
    pub(crate) const fn is_zero(&self) -> bool {
        self.hi == 0 && self.lo == 0
    }

    /// Returns the number of leading zeros in the binary representation of
    /// `self`.
    pub(crate) const fn leading_zeros(&self) -> u32 {
        self.hi.leading_zeros()
            + (self.hi == 0) as u32 * self.lo.leading_zeros()
    }

    /// Returns the number of trailing zeros in the binary representation of
    /// `self`.
    pub(crate) const fn trailing_zeros(&self) -> u32 {
        self.lo.trailing_zeros()
            + (self.lo == 0) as u32 * self.hi.trailing_zeros()
    }

    /// Returns the index of the most significant bit of `self`.
    /// Pre-condition: `self` must not be zero!
    pub(crate) fn msb(&self) -> u32 {
        debug_assert!(!self.is_zero());
        Self::BITS - self.leading_zeros() - 1
    }

    /// Add 1 to `self` inplace.
    #[inline]
    pub(crate) fn incr(&mut self) {
        self.lo = self.lo.wrapping_add(1_u128);
        self.hi = self.hi.wrapping_add((self.lo == 0) as u128);
    }

    /// Multiply by 10 and add decimal digit (inplace).
    pub(crate) fn imul10_add(&mut self, d: u8) {
        debug_assert!(
            *self
                <= u256::new(
                    0x19999999999999999999999999999999_u128,
                    0x99999999999999999999999999999999_u128
                )
        );
        debug_assert!(d < 10);
        let ll = u128_lo(self.lo);
        let lh = u128_hi(self.lo);
        let hl = u128_lo(self.hi);
        let hh = u128_hi(self.hi);
        let mut t = ll * 10 + d as u128;
        self.lo = u128_lo(t);
        t = lh * 10 + u128_hi(t);
        self.lo += t << 64;
        t = hl * 10 + u128_hi(t);
        self.hi = u128_lo(t);
        t = hh * 10 + u128_hi(t);
        self.hi += t << 64;
    }

    /// Divide `self` inplace by `2^p` and round (tie to even).
    pub(crate) fn idiv_pow2(&mut self, mut p: u32) {
        debug_assert_ne!(p, 0);
        debug_assert!(p < Self::BITS);
        if p > 128 {
            p -= 128;
            let tie = 1 << (p - 1);
            let hi_rem = self.hi & ((1 << p) - 1);
            self.lo = self.hi >> p;
            self.hi = 0;
            if hi_rem > tie || (hi_rem == tie && (self.lo & 1_u128) == 1) {
                self.incr();
            }
        } else {
            let tie = 1 << (p - 1);
            let lo_rem = self.hi & ((1 << p) - 1);
            self.lo >>= p;
            self.lo |= self.hi << (128 - p);
            self.hi >>= p;
            if lo_rem > tie || (lo_rem == tie && (self.lo & 1_u128) == 1) {
                self.incr();
            }
        }
    }

    #[cfg(target_endian = "big")]
    #[allow(clippy::cast_possible_truncation)]
    #[inline]
    /// Raw transmutation to `[u64; 4]` (in native endian order).
    pub(crate) const fn to_bits(&self) -> [u64; 4] {
        [
            u128_hi(self.hi) as u64,
            u128_lo(self.hi) as u64,
            u128_hi(self.lo) as u64,
            u128_lo(self.lo) as u64,
        ]
    }

    #[cfg(target_endian = "little")]
    #[allow(clippy::cast_possible_truncation)]
    #[inline]
    /// Raw transmutation to `[u64; 4]` (in native endian order).
    pub(crate) const fn to_bits(self) -> [u64; 4] {
        [
            u128_lo(self.lo) as u64,
            u128_hi(self.lo) as u64,
            u128_lo(self.hi) as u64,
            u128_hi(self.hi) as u64,
        ]
    }

    #[cfg(target_endian = "big")]
    #[inline]
    /// Raw transmutation from `[u64; 4]` (in native endian order).
    pub(crate) const fn from_bits(bits: [u64; 4]) -> Self {
        Self {
            hi: (bits[0] as u128) << 64 | (bits[1] as u128),
            lo: (bits[2] as u128) << 64 | (bits[3] as u128),
        }
    }

    #[cfg(target_endian = "little")]
    #[inline]
    /// Raw transmutation from `[u64; 4]` (in native endian order).
    pub(crate) const fn from_bits(bits: [u64; 4]) -> Self {
        Self {
            hi: (bits[3] as u128) << 64 | (bits[2] as u128),
            lo: (bits[1] as u128) << 64 | (bits[0] as u128),
        }
    }

    pub(crate) const fn shl(self, rhs: u32) -> Self {
        const LIMIT: u32 = u256::BITS - 1;
        assert!(rhs <= LIMIT, "Attempt to shift left with overflow.");
        match rhs {
            1..=127 => Self {
                hi: self.hi << rhs | self.lo >> (128 - rhs),
                lo: self.lo << rhs,
            },
            128 => Self { hi: self.lo, lo: 0 },
            129..=255 => Self {
                hi: self.lo << (rhs - 128),
                lo: 0,
            },
            0 => self,
            _ => unreachable!(),
        }
    }
}

impl AddAssign<&Self> for u256 {
    fn add_assign(&mut self, rhs: &Self) {
        // TODO: change when [feature(bigint_helper_methods)] got stable
        // let mut carry = false;
        // (self.lo, carry) = self.lo.carrying_add(other.lo, carry);
        // (self.hi, carry) = self.hi.carrying_add(other.lo, carry);
        self.lo = self.lo.wrapping_add(rhs.lo);
        self.hi = self.hi.wrapping_add((self.lo < rhs.lo) as u128);
        self.hi = self.hi.wrapping_add(rhs.hi);
    }
}

impl AddAssign<u128> for u256 {
    fn add_assign(&mut self, rhs: u128) {
        self.lo = self.lo.wrapping_add(rhs);
        self.hi = self.hi.wrapping_add((self.lo < rhs) as u128);
    }
}

impl SubAssign<&Self> for u256 {
    fn sub_assign(&mut self, rhs: &Self) {
        // TODO: change when [feature(bigint_helper_methods)] got stable
        // let mut borrow = false;
        // (self.lo, borrow) = self.lo.borrowing_sub(rhs.lo, borrow);
        // (self.hi, borrow) = self.hi.borrowing_sub(rhs.lo, borrow);
        let t = self.lo.wrapping_sub(rhs.lo);
        self.hi = self
            .hi
            .wrapping_sub(rhs.hi)
            .wrapping_sub((t > self.lo) as u128);
        self.lo = t;
    }
}

impl SubAssign<u128> for u256 {
    fn sub_assign(&mut self, rhs: u128) {
        let t = self.lo.wrapping_sub(rhs);
        self.hi = self.hi.wrapping_sub((t > self.lo) as u128);
        self.lo = t;
    }
}

impl MulAssign<u128> for u256 {
    fn mul_assign(&mut self, rhs: u128) {
        let tl = u128_widening_mul(self.lo, rhs);
        self.lo = tl.lo;
        let th = u128_widening_mul(self.hi, rhs);
        self.hi = th.lo.wrapping_add(tl.hi);
    }
}

impl Shl<u32> for u256 {
    type Output = Self;

    fn shl(self, rhs: u32) -> Self::Output {
        self.shl(rhs)
    }
}

impl ShlAssign<u32> for u256 {
    fn shl_assign(&mut self, rhs: u32) {
        const LIMIT: u32 = u256::BITS - 1;
        assert!(rhs <= LIMIT, "Attempt to shift left with overflow.");
        match rhs {
            1..=127 => {
                self.hi <<= rhs;
                self.hi |= self.lo >> (128 - rhs);
                self.lo <<= rhs;
            }
            128 => {
                self.hi = self.lo;
                self.lo = 0;
            }
            129..=255 => {
                self.hi = self.lo << (rhs - 128);
                self.lo = 0;
            }
            0 => {}
            _ => unreachable!(),
        }
    }
}

impl Shr<u32> for u256 {
    type Output = Self;

    fn shr(self, rhs: u32) -> Self::Output {
        const LIMIT: u32 = u256::BITS - 1;
        assert!(rhs <= LIMIT, "Attempt to shift right with overflow.");
        match rhs {
            1..=127 => Self::Output {
                hi: self.hi >> rhs,
                lo: self.hi << (128 - rhs) | self.lo >> rhs,
            },
            128 => Self::Output { hi: 0, lo: self.hi },
            129..=255 => Self::Output {
                hi: 0,
                lo: self.hi >> (rhs - 128),
            },
            0 => self,
            _ => unreachable!(),
        }
    }
}

impl ShrAssign<u32> for u256 {
    fn shr_assign(&mut self, rhs: u32) {
        const LIMIT: u32 = u256::BITS - 1;
        assert!(rhs <= LIMIT, "Attempt to shift right with overflow.");
        match rhs {
            1..=127 => {
                self.lo >>= rhs;
                self.lo |= self.hi << (128 - rhs);
                self.hi >>= rhs;
            }
            128 => {
                self.lo = self.hi;
                self.hi = 0;
            }
            129..=255 => {
                self.lo = self.hi >> (rhs - 128);
                self.hi = 0;
            }
            0 => {}
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod u256_shift_tests {
    use crate::uint256::u256;

    #[test]
    fn test_shl() {
        let u = u256 {
            hi: u128::MAX,
            lo: u128::MAX,
        };
        assert_eq!(u << 0, u);
        let v = u << 7;
        assert_eq!(
            v,
            u256 {
                hi: u128::MAX,
                lo: u.lo << 7,
            }
        );
        let v = u << 128;
        assert_eq!(
            v,
            u256 {
                hi: u128::MAX,
                lo: 0,
            }
        );
        let v = u << 132;
        assert_eq!(
            v,
            u256 {
                hi: u128::MAX << 4,
                lo: 0,
            }
        );
        let v = u << 255;
        assert_eq!(
            v,
            u256 {
                hi: 1 << 127,
                lo: 0,
            }
        );
    }

    #[test]
    fn test_shr() {
        let u = u256 {
            hi: u128::MAX,
            lo: u128::MAX,
        };
        assert_eq!(u >> 0, u);
        let v = u >> 3;
        assert_eq!(
            v,
            u256 {
                hi: u.hi >> 3,
                lo: u128::MAX,
            }
        );
        let v = u >> 128;
        assert_eq!(
            v,
            u256 {
                hi: 0,
                lo: u128::MAX,
            }
        );
        let v = u >> 140;
        assert_eq!(
            v,
            u256 {
                hi: 0,
                lo: u128::MAX >> 12,
            }
        );
    }

    #[test]
    fn test_shl_assign() {
        let o = u256 {
            hi: 0x23,
            lo: u128::MAX - 1,
        };
        let mut u = o;
        u <<= 0;
        assert_eq!(u, o);
        u <<= 4;
        assert_eq!(
            u,
            u256 {
                hi: 0x23f,
                lo: 0xffffffffffffffffffffffffffffffe0,
            }
        );
        u <<= 128;
        assert_eq!(
            u,
            u256 {
                hi: 0xffffffffffffffffffffffffffffffe0,
                lo: 0
            }
        );
        u <<= 133;
        assert_eq!(u, u256 { hi: 0, lo: 0 });
    }

    #[test]
    fn test_shr_assign() {
        let o = u256 {
            hi: u128::MAX - 25,
            lo: u128::MAX - 1,
        };
        let mut u = o;
        u >>= 0;
        assert_eq!(u, o);
        u >>= 27;
        assert_eq!(
            u,
            u256 {
                hi: 0x1fffffffffffffffffffffffff,
                lo: 0xfffffcdfffffffffffffffffffffffff,
            }
        );
        u >>= 128;
        assert_eq!(
            u,
            u256 {
                hi: 0,
                lo: 0x1fffffffffffffffffffffffff
            }
        );
        u = o;
        u >>= 255;
        assert_eq!(u, u256 { hi: 0, lo: 1 });
    }
}
