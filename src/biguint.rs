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
    fmt,
    mem::size_of,
    ops::{
        Add, AddAssign, BitOrAssign, MulAssign, Rem, Shl, ShlAssign, Shr,
        ShrAssign, Sub, SubAssign,
    },
};

const CHUNK_SIZE: u32 = 19;
const CHUNK_BASE: u64 = 10_u64.pow(CHUNK_SIZE);

#[inline(always)]
const fn u128_hi(u: u128) -> u128 {
    u >> 64
}

#[inline(always)]
const fn u128_lo(u: u128) -> u128 {
    u & 0xffffffffffffffff
}

#[inline(always)]
const fn u128_divrem(x: u128, y: u128) -> (u128, u128) {
    (x / y, x % y)
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

// Calculate ⌊(x * y) / 2⁵¹²⌋.
pub(crate) fn u256_truncating_mul_u512(x: &u256, y: &u512) -> u256 {
    let mut t = u256::new(0, u128_widening_mul(x.lo, y.lo.lo).hi);
    t += &u128_widening_mul(x.hi, y.lo.lo);
    let mut u = u128_widening_mul(x.lo, y.lo.hi);
    t += &u;
    t = u256::new((t < u) as u128, t.hi);
    t += &u128_widening_mul(x.hi, y.lo.hi);
    u = u128_widening_mul(x.lo, y.hi.lo);
    t += &u;
    t = u256::new((t < u) as u128, t.hi);
    t += &u128_widening_mul(x.hi, y.hi.lo);
    u = u128_widening_mul(x.lo, y.hi.hi);
    t += &u;
    t = u256::new((t < u) as u128, t.hi);
    t += &u128_widening_mul(x.hi, y.hi.hi);
    t
}

pub(crate) trait DivRem<RHS = Self> {
    type Output;
    fn div_rem(self, rhs: RHS) -> Self::Output;
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

    /// Multiplicative identity = 1.
    pub(crate) const ONE: Self = Self::new(0, 1);

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

    /// Subtract 1 from `self` inplace.
    #[inline]
    pub(crate) fn decr(&mut self) {
        self.hi = self.hi.wrapping_sub((self.lo == 0) as u128);
        self.lo = self.lo.wrapping_sub(1);
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

    /// Divide `self` inplace by 2ⁿ and round (tie to even).
    pub(crate) fn idiv_pow2(&mut self, mut n: u32) {
        debug_assert_ne!(n, 0);
        debug_assert!(n < Self::BITS);
        let tie = &Self::ONE << (n - 1);
        let rem = self.rem_pow2(n);
        *self >>= n;
        if rem > tie || (rem == tie && (self.lo & 1) == 1) {
            self.incr();
        }
    }

    // Specialized version adapted from
    // Henry S. Warren, Hacker’s Delight,
    // originally found at http://www.hackersdelight.org/HDcode/divlu.c.txt.
    // That code is in turn based on Algorithm D from
    // D. E. Knuth, The Art of Computer Programming, Vol. 2, Ch. 4.3.1,
    // adapted to the special case m = 4 and n = 2 and HI(x) < y (!).
    // The link given above does not exist anymore, but the code can still be
    // found at https://github.com/hcs0/Hackers-Delight/blob/master/divlu.c.txt.
    /// Returns `self` / rhs, `self` % rhs
    fn div_rem_u128_special(&self, rhs: u128) -> (Self, u128) {
        debug_assert!(self.hi < rhs);
        const B: u128 = 1 << 64;
        // Normalize dividend and divisor, so that the divisor has its highest
        // bit set, and get their 64-bit parts.
        let shift = rhs.leading_zeros();
        let x = self << shift;
        let x32 = x.hi;
        let x1 = u128_hi(x.lo);
        let x0 = u128_lo(x.lo);
        let y = rhs << shift;
        let y1 = u128_hi(y);
        let y0 = u128_lo(y);

        let (mut q1, mut rhat) = u128_divrem(x32, y1);
        // Now we have
        // q1 * y1 + rhat = x32
        // so that
        // q1 * y1 * 2⁶⁴ + rhat * 2⁶⁴ + x1 = x32 * 2⁶⁴ + x1
        while q1 >= B || q1 * y0 > rhat * B + x1 {
            q1 -= 1;
            rhat += y1;
            if rhat >= B {
                break;
            }
        }
        // The loop did not change the equation given above. It was terminated
        // if either q1 < 2⁶⁴ or rhat >= 2⁶⁴ or q1 * yn0 > rhat * 2⁶⁴ + x1.
        // In these cases follows:
        // q1 * y0 <= rhat * 2⁶⁴ + x1, therefor
        // q1 * y1 * 2⁶⁴ + q1 * y0 <= x32 * 2⁶⁴ + x1, and
        // q1 * y <= x32 * 2⁶⁴ + x1, and
        // x32 * 2⁶⁴ + x1 - q1 * y >= 0.
        // That means that the add-back step in Knuth's algorithm D is not
        // required.

        // Since the final quotient is < 2¹²⁸, this must also be true for
        // x32 * 2⁶⁴ + x1 - q1 * y. Thus, in the following we can safely
        // ignore any possible overflow in x32 * 2⁶⁴ or q1 * y.
        let t = x32.wrapping_shl(64) + x1 - q1.wrapping_mul(y);
        let (mut q0, mut rhat) = u128_divrem(t, y1);
        while q0 >= B || q0 * y0 > rhat * B + x0 {
            q0 -= 1;
            rhat += y1;
            if rhat >= B {
                break;
            }
        }
        // q = q1 * B + q0
        let q = (q1 << 64) + q0;
        // Denormalize remainder
        let r = (t.wrapping_shl(64) + x0 - q0.wrapping_mul(y)) >> shift;
        (u256::new(0_u128, q), r)
    }

    /// Returns `self` / `rhs`, rounded tie to even.
    pub(crate) fn div_rounded(&self, rhs: &Self) -> Self {
        let (mut quot, rem) = self.div_rem(rhs);
        let tie = rhs >> 1;
        if rem > tie || (rem == tie && (quot.lo & 1) == 1) {
            quot.incr();
        }
        quot
    }

    /// Returns `self` / 10ⁿ, rounded tie to even.
    pub(crate) fn div_pow10_rounded(&self, n: u32) -> Self {
        let mut q = *self;
        let mut r = 0_u64;
        if n <= CHUNK_SIZE {
            let d = 10_u64.pow(n);
            (q, r) = q.div_rem(d);
            let tie = d >> 1;
            if r > tie || (r == tie && (q.lo & 1) == 1) {
                q.incr();
            }
        } else {
            let n = (n - 1) / CHUNK_SIZE;
            let mut all_chunks_zero = true;
            for _ in 0..n {
                (q, r) = q.div_rem(CHUNK_BASE);
                all_chunks_zero = all_chunks_zero && r == 0;
            }
            let d = 10_u64.pow(n % CHUNK_SIZE);
            (q, r) = q.div_rem(d);
            let tie = d >> 1;
            if r > tie || (r == tie && (q.lo & 1) == 1 && all_chunks_zero) {
                q.incr();
            }
        }
        q
    }

    /// Returns `self` % 2ⁿ, i.e. the n left-most bits of self.
    pub(crate) fn rem_pow2(&self, n: u32) -> Self {
        match n {
            0 => u256::ZERO,
            1..=127 => u256::new(0, self.lo & ((1 << n) - 1)),
            128..=255 => u256::new(self.hi & ((1 << (n - 128)) - 1), self.lo),
            _ => *self,
        }
    }

    // TODO: remove this function and replaced calls to it by op <<
    // when trait fns can be declared const.
    pub(crate) const fn shift_left(&self, rhs: u32) -> u256 {
        const LIMIT: u32 = u256::BITS - 1;
        assert!(rhs <= LIMIT, "Attempt to shift left with overflow.");
        match rhs {
            1..=127 => u256 {
                hi: self.hi << rhs | self.lo >> (128 - rhs),
                lo: self.lo << rhs,
            },
            128 => u256 { hi: self.lo, lo: 0 },
            129..=255 => u256 {
                hi: self.lo << (rhs - 128),
                lo: 0,
            },
            0 => *self,
            _ => unreachable!(),
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

    // Calculate z = x * y.
    pub(crate) fn widening_mul(&self, rhs: &u256) -> u512 {
        let mut lo = u128_widening_mul(self.lo, rhs.lo);
        let mut t1 = u128_widening_mul(self.lo, rhs.hi);
        let mut t2 = u128_widening_mul(self.hi, rhs.lo);
        let mut hi = u128_widening_mul(self.hi, rhs.hi);
        t1 += &t2;
        hi += t1.hi;
        hi.hi += (t1 < t2) as u128;
        t2 = u256::new(t1.lo, 0);
        lo += &t2;
        hi += (lo < t2) as u128;
        u512 { hi, lo }
    }

    // Calculate ⌊(x * y) / 2²⁵⁶⌋.
    pub(crate) fn truncating_mul(&self, rhs: &u256) -> u256 {
        let mut r = u128_widening_mul(self.hi, rhs.hi);
        let t1 = u128_widening_mul(self.hi, rhs.lo);
        r += t1.hi;
        let t2 = u128_widening_mul(self.lo, rhs.hi);
        r += t2.hi;
        let mut c = 0_u128;
        let t3 = t1.lo.wrapping_add(t2.lo);
        c += (t3 < t1.lo) as u128;
        let t4 = t3.wrapping_add(u128_widening_mul(self.lo, rhs.lo).hi);
        c += (t4 < t3) as u128;
        r += c;
        r
    }
}

impl Add for &u256 {
    type Output = u256;

    fn add(self, rhs: Self) -> Self::Output {
        // TODO: change when [feature(bigint_helper_methods)] got stable
        // let (lo, carry) = self.lo.carrying_add(rhs.lo, false);
        // let (hi, _) = self.hi.carrying_add(rhs.hi, carry);
        let lo = self.lo.wrapping_add(rhs.lo);
        let hi = self
            .hi
            .wrapping_add(rhs.hi)
            .wrapping_add((lo < rhs.lo) as u128);
        Self::Output { hi, lo }
    }
}

impl Add<u32> for &u256 {
    type Output = u256;

    fn add(self, rhs: u32) -> Self::Output {
        // TODO: change when [feature(bigint_helper_methods)] got stable
        // let (lo, carry) = self.lo.carrying_add(rhs, false);
        // let hi = self.hi.wrapping_add(carry);
        let lo = self.lo.wrapping_add(rhs as u128);
        let hi = self.hi.wrapping_add((lo < self.lo) as u128);
        Self::Output { hi, lo }
    }
}

impl AddAssign<&Self> for u256 {
    fn add_assign(&mut self, rhs: &Self) {
        // TODO: change when [feature(bigint_helper_methods)] got stable
        // let mut carry = false;
        // (self.lo, carry) = self.lo.carrying_add(rhs.lo, carry);
        // (self.hi, carry) = self.hi.carrying_add(rhs.hi, carry);
        self.lo = self.lo.wrapping_add(rhs.lo);
        self.hi = self
            .hi
            .wrapping_add(rhs.hi)
            .wrapping_add((self.lo < rhs.lo) as u128);
    }
}

impl AddAssign<u128> for u256 {
    fn add_assign(&mut self, rhs: u128) {
        self.lo = self.lo.wrapping_add(rhs);
        self.hi = self.hi.wrapping_add((self.lo < rhs) as u128);
    }
}

impl Sub for &u256 {
    type Output = u256;

    fn sub(self, rhs: Self) -> Self::Output {
        // TODO: change when [feature(bigint_helper_methods)] got stable
        // let (lo, borrow) = self.lo.borrowing_sub(rhs.lo, false);
        // let (hi, _) = self.hi.borrowing_sub(rhs.hi, borrow);
        let lo = self.lo.wrapping_sub(rhs.lo);
        let hi = self
            .hi
            .wrapping_sub(rhs.hi)
            .wrapping_sub((lo > self.lo) as u128);
        Self::Output { hi, lo }
    }
}

impl Sub<u32> for &u256 {
    type Output = u256;

    fn sub(self, rhs: u32) -> Self::Output {
        // TODO: change when [feature(bigint_helper_methods)] got stable
        // let (lo, borrow) = self.lo.borrowing_add(rhs, false);
        // let hi = self.hi.wrapping_add(borrow);
        let lo = self.lo.wrapping_sub(rhs as u128);
        let hi = self.hi.wrapping_sub((lo > self.lo) as u128);
        Self::Output { hi, lo }
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

impl DivRem<u64> for &u256 {
    type Output = (u256, u64);

    /// Returns `self` / rhs, `self` % rhs
    fn div_rem(self, rhs: u64) -> Self::Output {
        let (quot_hi, r) = u128_divrem(self.hi, rhs as u128);
        let (mut quot_lo, r) =
            u128_divrem((r << 64) + u128_hi(self.lo), rhs as u128);
        quot_lo <<= 64;
        let (t, r) = u128_divrem((r << 64) + u128_lo(self.lo), rhs as u128);
        quot_lo += t;
        (u256::new(quot_hi, quot_lo), r as u64)
    }
}

impl DivRem<u128> for &u256 {
    type Output = (u256, u128);

    /// Returns `self` / rhs, `self` % rhs
    fn div_rem(self, rhs: u128) -> Self::Output {
        if self.hi == 0 {
            (u256::new(0_u128, self.lo / rhs), self.lo % rhs)
        } else if u128_hi(rhs) == 0 {
            let (quot, rem) = self.div_rem(u128_lo(rhs) as u64);
            (quot, rem as u128)
        } else if self.hi < rhs {
            self.div_rem_u128_special(rhs)
        } else {
            let mut quot = *self;
            let mut rem = 0_u128;
            quot.hi %= rhs;
            (quot, rem) = quot.div_rem_u128_special(rhs);
            quot.hi = self.hi / rhs;
            (quot, rem)
        }
    }
}

impl DivRem<&u256> for &u256 {
    type Output = (u256, u256);

    /// Returns `self` / rhs, `self` % rhs
    fn div_rem(self, rhs: &u256) -> Self::Output {
        if rhs.hi == 0 {
            let (quot, rem) = self.div_rem(rhs.lo);
            (quot, u256::new(0, rem))
        } else {
            let mut quot = self.hi / rhs.hi;
            let mut t = *rhs;
            t *= quot;
            if t > *self {
                t -= rhs;
                quot -= 1
            } else if (&t + rhs) < *self {
                t += rhs;
                quot += 1
            }
            let rem = self - &t;
            (u256::new(0, quot), rem)
        }
    }
}

impl Rem<u64> for &u256 {
    type Output = u64;

    #[inline]
    fn rem(self, rhs: u64) -> Self::Output {
        (self % rhs as u128) as u64
    }
}

impl Rem<u128> for &u256 {
    type Output = u128;

    #[inline]
    fn rem(self, rhs: u128) -> Self::Output {
        let mut rem = self.hi % rhs;
        rem = ((rem << 64) + u128_hi(self.lo)) % rhs;
        rem = ((rem << 64) + u128_lo(self.lo)) % rhs;
        rem
    }
}

impl BitOrAssign for u256 {
    fn bitor_assign(&mut self, rhs: Self) {
        self.hi |= rhs.hi;
        self.lo |= rhs.lo;
    }
}

impl Shl<u32> for &u256 {
    type Output = u256;

    fn shl(self, rhs: u32) -> Self::Output {
        assert!(
            rhs <= (Self::Output::BITS - 1),
            "Attempt to shift left with overflow."
        );
        match rhs {
            1..=127 => Self::Output {
                hi: self.hi << rhs | self.lo >> (128 - rhs),
                lo: self.lo << rhs,
            },
            128 => Self::Output { hi: self.lo, lo: 0 },
            129..=255 => Self::Output {
                hi: self.lo << (rhs - 128),
                lo: 0,
            },
            0 => self.clone(),
            _ => unreachable!(),
        }
    }
}

impl ShlAssign<u32> for u256 {
    fn shl_assign(&mut self, rhs: u32) {
        assert!(
            rhs <= (Self::BITS - 1),
            "Attempt to shift left with overflow."
        );
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

impl Shr<u32> for &u256 {
    type Output = u256;

    fn shr(self, rhs: u32) -> Self::Output {
        assert!(
            rhs <= (Self::Output::BITS - 1),
            "Attempt to shift right with overflow."
        );
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
            0 => self.clone(),
            _ => unreachable!(),
        }
    }
}

impl ShrAssign<u32> for u256 {
    fn shr_assign(&mut self, rhs: u32) {
        assert!(
            rhs <= (Self::BITS - 1),
            "Attempt to shift right with overflow."
        );
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

impl fmt::Display for u256 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const SEGMENT_BASE: u64 = 1_000_000_000_000_000_000;
        if self.hi == 0 {
            return fmt::Display::fmt(&self.lo, f);
        }
        let mut segments: [u64; 5] = [0, 0, 0, 0, 0];
        let mut t = *self;
        let mut r = 0_u64;
        let mut idx = 0;
        while !t.is_zero() {
            (t, r) = t.div_rem(SEGMENT_BASE);
            segments[idx] = r;
            idx += 1;
        }
        idx -= 1;
        write!(f, "{}", segments[idx]);
        while idx > 0 {
            idx -= 1;
            write!(f, "{:018}", segments[idx]);
        }
        Ok(())
    }
}

/// Helper type representing unsigned integers of 512 bits.
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialOrd, PartialEq)]
pub(crate) struct u512 {
    pub(crate) hi: u256,
    pub(crate) lo: u256,
}

impl u512 {
    /// The size of this integer type in bits.
    #[allow(clippy::cast_possible_truncation)]
    pub(crate) const BITS: u32 = size_of::<Self>() as u32 * 8;

    /// Additive identity = 0.
    pub(crate) const ZERO: Self = Self::new(u256::ZERO, u256::ZERO);

    /// Multiplicative identity = 1.
    pub(crate) const ONE: Self = Self::new(u256::ZERO, u256::ONE);

    /// Maximum value = 2⁵¹² - 1.
    pub(crate) const MAX: Self = Self::new(u256::MAX, u256::MAX);

    /// Create an `u256` value from two u256 values.
    #[inline(always)]
    pub(crate) const fn new(hi: u256, lo: u256) -> Self {
        Self { hi, lo }
    }

    /// Return true, if `self` == 0.
    #[inline]
    pub(crate) const fn is_zero(&self) -> bool {
        self.hi.is_zero() && self.lo.is_zero()
    }

    /// Add 1 to `self` inplace.
    #[inline]
    pub(crate) fn incr(&mut self) {
        self.lo.incr();
        if self.lo.is_zero() {
            self.incr();
        }
    }

    /// Divide `self` inplace by 2ⁿ and round (tie to even).
    pub(crate) fn idiv_pow2(&mut self, mut n: u32) {
        debug_assert_ne!(n, 0);
        debug_assert!(n < u256::BITS);
        let tie = &u256::ONE << (n - 1);
        let rem = self.lo.rem_pow2(n);
        *self >>= n;
        if rem > tie || (rem == tie && (self.lo.lo & 1) == 1) {
            self.incr();
        }
    }

    /// Returns `self` / 10ⁿ, rounded tie to even.
    pub(crate) fn div_pow10_rounded(&self, n: u32) -> Self {
        const CHUNK_SIZE: u32 = 38;
        const CHUNK_BASE: u128 = 10_u128.pow(CHUNK_SIZE);
        let mut q = *self;
        let mut r = 0_u128;
        if n <= CHUNK_SIZE {
            let d = 10_u128.pow(n);
            (q, r) = q.div_rem(d);
            let tie = d >> 1;
            if r > tie || (r == tie && (q.lo.lo & 1) == 1) {
                q.incr();
            }
        } else {
            let n = (n - 1) / CHUNK_SIZE;
            let mut all_chunks_zero = true;
            for _ in 0..n {
                (q, r) = q.div_rem(CHUNK_BASE);
                all_chunks_zero = all_chunks_zero && r == 0;
            }
            let d = 10_u128.pow(n % CHUNK_SIZE);
            (q, r) = q.div_rem(d);
            let tie = d >> 1;
            if r > tie || (r == tie && (q.lo.lo & 1) == 1 && all_chunks_zero) {
                q.incr();
            }
        }
        q
    }
}

impl DivRem<u128> for &u512 {
    type Output = (u512, u128);

    /// Returns `self` / rhs, `self` % rhs
    fn div_rem(self, rhs: u128) -> Self::Output {
        let (quot_hi, mut rem) = self.hi.div_rem(rhs);
        let mut t = u256::new(rem, self.lo.hi);
        (t, rem) = t.div_rem(rhs);
        debug_assert_eq!(t.hi, 0);
        let mut quot_lo = u256::new(t.lo, 0); // t << 128
        t = u256::new(rem, self.lo.lo);
        (t, rem) = t.div_rem(rhs);
        quot_lo += &t;
        (u512::new(quot_hi, quot_lo), rem)
    }
}

impl Rem<u64> for &u512 {
    type Output = u64;

    #[inline(always)]
    fn rem(self, rhs: u64) -> Self::Output {
        (self % rhs as u128) as u64
    }
}

impl Rem<u128> for &u512 {
    type Output = u128;

    #[inline]
    fn rem(self, rhs: u128) -> Self::Output {
        let mut rem = &self.hi % rhs;
        rem = &u256::new(rem, self.lo.hi) % rhs;
        rem = &u256::new(rem, self.lo.lo) % rhs;
        rem
    }
}

impl ShlAssign<u32> for u512 {
    fn shl_assign(&mut self, rhs: u32) {
        assert!(
            rhs <= (Self::BITS - 1),
            "Attempt to shift left with overflow."
        );
        match rhs {
            1..=255 => {
                self.hi <<= rhs;
                self.hi |= &self.lo >> (256 - rhs);
                self.lo <<= rhs;
            }
            256 => {
                self.hi = self.lo;
                self.lo = u256::ZERO;
            }
            257..=511 => {
                self.hi = &self.lo << (rhs - 256);
                self.lo = u256::ZERO;
            }
            0 => {}
            _ => unreachable!(),
        }
    }
}

impl ShrAssign<u32> for u512 {
    fn shr_assign(&mut self, rhs: u32) {
        assert!(
            rhs <= (Self::BITS - 1),
            "Attempt to shift left with overflow."
        );
        let mut k = rhs;
        match k {
            0..=127 => {
                let m = (1 << k) - 1;
                self.lo.lo =
                    (self.lo.lo >> k) | ((self.lo.hi & m) << (128 - k));
                self.lo.hi =
                    (self.lo.hi >> k) | ((self.hi.lo & m) << (128 - k));
                self.hi.lo =
                    (self.hi.lo >> k) | ((self.hi.hi & m) << (128 - k));
                self.hi.hi >>= k;
            }
            128 => {
                self.lo.lo = self.lo.hi;
                self.lo.hi = self.hi.lo;
                self.hi.lo = self.hi.hi;
                self.hi.hi = 0;
            }
            129..=255 => {
                k -= 128;
                let m = (1 << k) - 1;
                self.lo.lo =
                    (self.lo.hi >> k) | ((self.hi.lo & m) << (128 - k));
                self.lo.hi =
                    (self.hi.lo >> k) | ((self.hi.hi & m) << (128 - k));
                self.hi.lo = self.hi.hi >> k;
                self.hi.hi = 0;
            }
            256 => {
                self.lo.lo = self.hi.lo;
                self.lo.hi = self.hi.hi;
                self.hi.lo = 0;
                self.hi.hi = 0;
            }
            257..=383 => {
                k -= 256;
                let m = (1 << k) - 1;
                self.lo.lo =
                    (self.hi.lo >> k) | ((self.hi.hi & m) << (128 - k));
                self.lo.hi = self.hi.hi >> k;
                self.hi.lo = 0;
                self.hi.hi = 0;
            }
            384 => {
                self.lo.lo = self.hi.hi;
                self.lo.hi = 0;
                self.hi.lo = 0;
                self.hi.hi = 0;
            }
            _ => {
                self.lo.lo = self.hi.hi >> (k - 384);
                self.lo.hi = 0;
                self.hi.lo = 0;
                self.hi.hi = 0;
            }
        }
    }
}

impl fmt::Display for u512 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const SEGMENT_SIZE: usize = 38;
        const SEGMENT_BASE: u128 = 10_u128.pow(SEGMENT_SIZE as u32);
        if self.hi.is_zero() {
            return fmt::Display::fmt(&self.lo, f);
        }
        let mut segments: [u128; 5] = [0, 0, 0, 0, 0];
        let mut t = *self;
        let mut r = 0_u128;
        let mut idx = 0;
        while !t.is_zero() {
            (t, r) = t.div_rem(SEGMENT_BASE);
            segments[idx] = r;
            idx += 1;
        }
        idx -= 1;
        write!(f, "{}", segments[idx]);
        while idx > 0 {
            idx -= 1;
            write!(f, "{:0SEGMENT_SIZE$}", segments[idx]);
        }
        Ok(())
    }
}

#[cfg(test)]
mod u256_div_rem_tests {
    use super::*;

    #[test]
    fn test_div_rem() {
        let v = u256::new(0, u128::MAX - 1);
        assert_eq!(v.div_rem(u128::MAX), (u256::ZERO, v.lo));
        let v = u256::MAX;
        assert_eq!(
            v.div_rem(7000_u64),
            (
                u256::new(
                    48611766702991209066196372490252601,
                    216614032428528827598971035816565592892
                ),
                3935
            )
        );
        assert_eq!(
            v.div_rem(10_u128.pow(28)),
            (
                u256::new(34028236692, 31934256858593286117999845820724523012),
                564039457584007913129639935
            )
        );
        let v = u256::new(70299, 93425685859328611799984582072);
        assert_eq!(
            v.div_rem(10_u128.pow(27) + 3),
            (u256::new(0, 23921510112175146), 468697630784693143145201978)
        )
    }

    #[test]
    fn test_div_rem10() {
        let v = u256::ZERO;
        assert_eq!(v.div_rem(10_u64), (u256::ZERO, 0));
        let v = u256::new(0, 7);
        assert_eq!(v.div_rem(10_u64), (u256::ZERO, 7));
        let v = u256::MAX;
        assert_eq!(
            v.div_rem(10_u64),
            (
                u256::new(
                    34028236692093846346337460743176821145,
                    204169420152563078078024764459060926873
                ),
                5
            )
        );
    }

    #[test]
    fn test_div_rem_pow10() {
        let v = u256::ZERO;
        assert_eq!(v.div_rem(10_u64.pow(10)), (u256::ZERO, 0));
        let v = u256::new(0, 700003);
        assert_eq!(v.div_rem(10_u64.pow(5)), (u256::new(0, 7), 3));
        let v = u256::new(0, u128::MAX);
        assert_eq!(
            v.div_rem(10_u64.pow(18)),
            (
                u256::new(0, u128::MAX / 10_u128.pow(18)),
                (u128::MAX % 10_u128.pow(18)) as u64
            )
        );
        let v = u256::MAX;
        assert_eq!(
            v.div_rem(10_u64.pow(18)),
            (
                u256::new(
                    340282366920938463463,
                    127472303548260950450562498184250007329
                ),
                584007913129639935
            )
        );
    }
}

#[cfg(test)]
mod u256_to_str_tests {
    use super::*;

    #[test]
    fn test_zero() {
        let v = u256::ZERO;
        assert_eq!(v.to_string(), "0")
    }

    #[test]
    fn test_max() {
        let v = u256::MAX;
        assert_eq!(
            v.to_string(),
            "115792089237316195423570985008687907853269984665640564039457584007\
             913129639935"
        )
    }
}

#[cfg(test)]
mod u256_wrapping_mul_u512_tests {
    use super::*;

    #[test]
    fn test_max() {
        let x = u256::MAX;
        let y: u512 = u512 {
            hi: u256::MAX,
            lo: u256::MAX,
        };
        let z = u256_truncating_mul_u512(&x, &y);
        assert_eq!(z, u256::new(u128::MAX, u128::MAX - 1));
    }

    #[test]
    fn test_one_times_max() {
        let x = u256::new(0, 1);
        let y: u512 = u512 {
            hi: u256::MAX,
            lo: u256::MAX,
        };
        let z = u256_truncating_mul_u512(&x, &y);
        assert_eq!(z, u256::ZERO);
    }

    #[test]
    fn test_max_plus_one_times_max() {
        let x = u256::new(1, 0);
        let y: u512 = u512 {
            hi: u256::MAX,
            lo: u256::MAX,
        };
        let z = u256_truncating_mul_u512(&x, &y);
        assert_eq!(z, u256::new(0, u128::MAX));
    }
}

#[cfg(test)]
mod u256_shift_tests {
    use super::*;

    #[test]
    fn test_u256_truncating_mul() {
        let x = u256::MAX;
        let y = u256::new(0, 1);
        let mut p = x.truncating_mul(&y);
        assert_eq!(p, u256::new(0, 0));
        let y = u256::new(0, 2);
        p = x.truncating_mul(&y);
        assert_eq!(p, u256::new(0, 1));
        p = x.truncating_mul(&x);
        p.incr();
        assert_eq!(p, x);
        let x = u256::new(1, u128::MAX);
        p = x.truncating_mul(&x);
        assert_eq!(p, u256::new(0, 3));
    }

    #[test]
    fn test_shl() {
        let u = u256 {
            hi: u128::MAX,
            lo: u128::MAX,
        };
        assert_eq!(&u << 0, u);
        let v = &u << 7;
        assert_eq!(
            v,
            u256 {
                hi: u128::MAX,
                lo: u.lo << 7,
            }
        );
        let v = &u << 128;
        assert_eq!(
            v,
            u256 {
                hi: u128::MAX,
                lo: 0,
            }
        );
        let v = &u << 132;
        assert_eq!(
            v,
            u256 {
                hi: u128::MAX << 4,
                lo: 0,
            }
        );
        let v = &u << 255;
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
        assert_eq!(&u >> 0, u);
        let v = &u >> 3;
        assert_eq!(
            v,
            u256 {
                hi: u.hi >> 3,
                lo: u128::MAX,
            }
        );
        let v = &u >> 128;
        assert_eq!(
            v,
            u256 {
                hi: 0,
                lo: u128::MAX,
            }
        );
        let v = &u >> 140;
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

#[cfg(test)]
mod u512_div_rem_tests {
    use super::*;

    #[test]
    fn test_div_rem() {
        let v = u512::MAX;
        assert_eq!(
            v.div_rem(2_u128),
            (u512::new(&u256::MAX >> 1, u256::MAX), 1)
        );
    }

    #[test]
    fn test_div_rem10() {
        let v = u512::ZERO;
        assert_eq!(v.div_rem(10_u128), (u512::ZERO, 0_u128));
        let v = u512::new(u256::ZERO, u256::new(0, 7));
        assert_eq!(v.div_rem(10_u128), (u512::ZERO, 7_u128));
        let v = u512::MAX;
        assert_eq!(
            v.div_rem(10_u128),
            (
                u512::new(
                    u256::new(
                        34028236692093846346337460743176821145,
                        204169420152563078078024764459060926873
                    ),
                    u256::new(
                        204169420152563078078024764459060926873,
                        204169420152563078078024764459060926873
                    )
                ),
                5
            )
        );
    }

    #[test]
    fn test_div_rem_pow10() {
        let v = u512::ZERO;
        assert_eq!(v.div_rem(10_u128.pow(10)), (u512::ZERO, 0));
        let v = u512::new(u256::ZERO, u256::new(2730, 490003));
        assert_eq!(
            v.div_rem(10_u128.pow(5)),
            (
                u512::new(
                    u256::ZERO,
                    u256::new(0, 9289708616941620052550126782887272177)
                ),
                64883
            )
        );
        let v = u512::new(u256::ZERO, u256::MAX);
        let d = 10_u128.pow(38);
        let (q, r) = u256::MAX.div_rem(d);
        assert_eq!(v.div_rem(d), (u512::new(u256::ZERO, q), r));
        let v = u512::MAX;
        assert_eq!(
            v.div_rem(10_u128.pow(27)),
            (
                u512::new(
                    u256::new(
                        340282366920,
                        319342568585932861179998458207245230120
                    ),
                    u256::new(
                        191932681663488487842607845281633842426,
                        86732842386697408091259742201350722586
                    )
                ),
                811946569946433649006084095
            )
        );
    }
}
