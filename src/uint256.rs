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
    ops::{Shl, ShlAssign, Shr, ShrAssign},
};

#[inline(always)]
const fn u128_hi(u: u128) -> u128 {
    u >> 64
}

#[inline(always)]
const fn u128_lo(u: u128) -> u128 {
    u & 0xffffffffffffffff
}

#[inline]
fn u128_mul_u128(x: u128, y: u128) -> u256 {
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

#[inline]
pub(crate) fn u256_mul(x: &u256, y: &u256) -> (u256, u64) {
    let mut t = u128_mul_u128(x.lo, y.lo);
    let mut rl = u256::new(0, t.lo);
    let mut c = u256::new(0, t.hi);
    t = u128_mul_u128(x.lo, y.hi);
    t.iadd(&c);
    let mut rh = u256::new(0, t.hi);
    c = u256::new(0, t.lo);
    t = u128_mul_u128(x.hi, y.lo);
    t.iadd(&c);
    rl.iadd(&u256::new(t.lo, 0));
    rh.iadd(&u128_mul_u128(x.hi, y.hi));
    rh.iadd(&u256::new(0, t.hi));
    let mut rem = u128_hi(rl.hi) as u64;
    rem |= (u128_lo(rl.hi) != 0 || rl.lo != 0) as u64;
    (rh, rem)
}

// Calculate x = x / y in place, where x = xh * 2^128 + xl, and return x % y.
// Adapted from
// D. E. Knuth, The Art of Computer Programming, Vol. 2, Ch. 4.3.1,
// Exercise 16
#[inline(always)]
fn u256_idiv_u64(xh: &mut u128, xl: &mut u128, y: u64) -> u128 {
    if y == 1 {
        return 0;
    }
    let y = y as u128;
    let mut th = u128_hi(*xh);
    let mut r = th % y;
    let mut tl = (r << 64) + u128_lo(*xh);
    *xh = ((th / y) << 64) + tl / y;
    r = tl % y;
    th = (r << 64) + u128_hi(*xl);
    r = th % y;
    tl = (r << 64) + u128_lo(*xl);
    *xl = ((th / y) << 64) + tl / y;
    tl % y
}

// Calculate x = x / y in place, where x = xh * 2^128 + xl, and return x % y.
// Specialized version adapted from
// Henry S. Warren, Hacker’s Delight,
// originally found at http://www.hackersdelight.org/HDcode/divlu.c.txt.
// That code is in turn based on Algorithm D from
// D. E. Knuth, The Art of Computer Programming, Vol. 2, Ch. 4.3.1,
// adapted to the special case m = 4 and n = 2 and xh < y (!).
// The link given above does not exist anymore, but the code can still be
// found at https://github.com/hcs0/Hackers-Delight/blob/master/divlu.c.txt.
#[inline(always)]
fn u256_idiv_u128_special(xh: &mut u128, xl: &mut u128, mut y: u128) -> u128 {
    debug_assert!(*xh < y);
    const B: u128 = 1 << 64;
    // Normalize dividend and divisor, so that y > 2^127 (i.e. highest bit set)
    let n_bits = y.leading_zeros();
    y <<= n_bits;
    let yn1 = u128_hi(y);
    let yn0 = u128_lo(y);
    // bits to be shifted from xl to xh:
    let sh = if n_bits == 0 {
        0
    } else {
        *xl >> (128 - n_bits)
    };
    let xn32 = *xh << n_bits | sh;
    let xn10 = *xl << n_bits;
    let xn1 = u128_hi(xn10);
    let xn0 = u128_lo(xn10);
    let mut q1 = xn32 / yn1;
    let mut rhat = xn32 % yn1;
    // Now we have
    // q1 * yn1 + rhat = xn32
    // so that
    // q1 * yn1 * 2^64 + rhat * 2^64 + xn1 = xn32 * 2^64 + xn1
    while q1 >= B || q1 * yn0 > rhat * B + xn1 {
        q1 -= 1;
        rhat += yn1;
        if rhat >= B {
            break;
        }
    }
    // The loop did not change the equation given above. It was terminated if
    // either q1 < 2^64 or rhat >= 2^64 or q1 * yn0 > rhat * 2^64 + xn1.
    // In these cases follows:
    // q1 * yn0 <= rhat * 2^64 + xn1, therefor
    // q1 * yn1 * 2^64 + q1 * yn0 <= xn32 * 2^64 + xn1, and
    // q1 * y <= xn32 * 2^64 + xn1, and
    // xn32 * 2^64 + xn1 - q1 * y >= 0.
    // That means that the add-back step in Knuth's algorithm is not required.
    // Since the final quotient is < 2^128, this must also be true for
    // xn32 * 2^64 + xn1 - q1 * y. Thus, in the following we can safely
    // ignore any possible overflow in xn32 * 2^64 or q1 * y.
    let t = xn32
        .wrapping_mul(B)
        .wrapping_add(xn1)
        .wrapping_sub(q1.wrapping_mul(y));
    let mut q0 = t / yn1;
    rhat = t % yn1;
    while q0 >= B || q0 * yn0 > rhat * B + xn0 {
        q0 -= 1;
        rhat += yn1;
        if rhat >= B {
            break;
        }
    }
    // Write back result
    *xh = 0;
    *xl = q1 * B + q0;
    // Denormalize remainder
    (t.wrapping_mul(B)
        .wrapping_add(xn0)
        .wrapping_sub(q0.wrapping_mul(y)))
        >> n_bits
}

// Calculate x = x / y in place, where x = xh * 2^128 + xl, and return x % y.
#[inline(always)]
fn u256_idiv_u128(xh: &mut u128, xl: &mut u128, y: u128) -> u128 {
    if u128_hi(y) == 0 {
        return u256_idiv_u64(xh, xl, u128_lo(y) as u64);
    }
    if *xh < y {
        return u256_idiv_u128_special(xh, xl, y);
    }
    let mut t = *xh % y;
    let r = u256_idiv_u128_special(&mut t, xl, y);
    *xh /= y;
    r
}

/// Return `Some<(q, r)>` with `q = (x * 10^p) / y` and `r = (x * 10^p) % y`,
/// so that `(x * 10^p) = q * y + r`, where q is rounded against floor so that
/// r, if non-zero, has the same sign as y and `0 <= abs(r) < abs(y)`, or return
/// `None` if |q| > i128::MAX.
// #[doc(hidden)]
// pub fn i128_shifted_div_mod_floor(
//     x: i128,
//     p: u8,
//     y: i128,
// ) -> Option<(i128, i128)> {
//     let (mut xh, mut xl) = u128_mul_u128(x.unsigned_abs(), ten_pow(p) as
// u128);     let r = u256_idiv_u128(&mut xh, &mut xl, y.unsigned_abs());
//     if xh != 0 || xl > i128::MAX as u128 {
//         return None;
//     }
//     // xl <= i128::MAX, so xl as i128 is safe.
//     let mut q = xl as i128;
//     // r < y, so r as i128 is safe.
//     let mut r = r as i128;
//     if x.is_negative() {
//         if y.is_negative() {
//             r = r.neg();
//         } else {
//             q = q.neg() - 1;
//             r = y - r;
//         }
//     } else if y.is_negative() {
//         q = q.neg() - 1;
//         r -= y;
//     }
//     Some((q, r))
// }

/// Return `Some<(q, r)>` with `q = (x1 * x2) / y` and `r = (x1 * x2) % y`,
/// so that `(x1 * x2) = q * y + r`, where q is rounded against floor so that
/// r, if non-zero, has the same sign as y and `0 <= abs(r) < abs(y)`, or return
/// `None` if |q| > i128::MAX.
// #[doc(hidden)]
// pub fn i256_div_mod_floor(x1: i128, x2: i128, y: i128) -> Option<(i128,
// i128)> {     debug_assert!(y > 0);
//     let (mut xh, mut xl) = u128_mul_u128(x1.unsigned_abs(),
// x2.unsigned_abs());     let r = u256_idiv_u128(&mut xh, &mut xl,
// y.unsigned_abs());     if xh != 0 || xl > i128::MAX as u128 {
//         return None;
//     }
//     // xl <= i128::MAX, so xl as i128 is safe.
//     let mut q = xl as i128;
//     // r < y, so r as i128 is safe.
//     let mut r = r as i128;
//     if x1.is_negative() != x2.is_negative() {
//         q = q.neg() - 1;
//         r = y - r;
//     }
//     Some((q, r))
// }

/// Helper type representing unsigned integers of 256 bits.
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, Eq, Ord, PartialOrd, PartialEq)]
pub(crate) struct u256 {
    pub(crate) hi: u128,
    pub(crate) lo: u128,
}

impl u256 {
    /// Create an `u256` value from two u128 values.
    #[inline(always)]
    pub(crate) const fn new(hi: u128, lo: u128) -> Self {
        Self { hi, lo }
    }

    /// The size of this integer type in bits.
    pub(crate) const BITS: u32 = size_of::<u256>() as u32 * 8;

    /// Return true, if `self` == 0.
    #[inline]
    pub(crate) const fn is_zero(&self) -> bool {
        self.hi == 0 && self.lo == 0
    }

    /// Returns the number of leading zeros in the binary representation of
    /// `self`.
    pub(crate) const fn leading_zeros(&self) -> u32 {
        return self.hi.leading_zeros()
            + (self.hi == 0) as u32 * self.lo.leading_zeros();
    }

    /// Returns the number of trailing zeros in the binary representation of
    /// `self`.
    pub(crate) const fn trailing_zeros(&self) -> u32 {
        return self.lo.trailing_zeros()
            + (self.lo == 0) as u32 * self.hi.trailing_zeros();
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

    /// Add `other` to `self` inplace.
    #[inline]
    pub(crate) fn iadd(&mut self, other: &u256) {
        self.lo = self.lo.wrapping_add(other.lo);
        self.hi = self.hi.wrapping_add((self.lo < other.lo) as u128);
        self.hi = self.hi.wrapping_add(other.hi);
    }

    // TODO: change when [feature(bigint_helper_methods)] got stable
    // pub(crate) fn iadd(&mut self, &other: u256) {
    //     let mut carry = false;
    //     (self.lo, carry) = self.lo.carrying_add(other.lo, carry);
    //     (self.hi, carry) = self.hi.carrying_add(other.lo, carry);
    // }

    /// Subtract `other` from `self` inplace.
    #[inline]
    pub(crate) fn isub(&mut self, other: &u256) {
        let t = self.lo.wrapping_sub(other.lo);
        self.hi = self
            .hi
            .wrapping_sub(other.hi)
            .wrapping_sub((t > self.lo) as u128);
        self.lo = t;
    }

    // TODO: change when [feature(bigint_helper_methods)] got stable
    // pub(crate) fn isub(&mut self, &other: u256) {
    //     let mut borrow = false;
    //     (self.lo, borrow) = self.lo.borrowing_add(other.lo, borrow);
    //     (self.hi, borrow) = self.hi.borrowing_add(other.lo, borrow);
    // }

    /// Divide `self` inplace by `2^p` and round (tie to even).
    pub(crate) fn idiv_pow2(&mut self, mut p: u32) {
        debug_assert_ne!(p, 0);
        debug_assert!(p < size_of::<u256>() as u32);
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
    #[inline]
    /// Raw transmutation to `[u64; 4]` (in native endian order).
    pub(crate) const fn to_bits(&self) -> [u64; 4] {
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

impl Default for u256 {
    fn default() -> Self {
        Self { hi: 0, lo: 0 }
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
