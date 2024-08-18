// ---------------------------------------------------------------------------
// Copyright:   (c) 2022 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use core::{
    fmt,
    mem::size_of,
    ops::{
        Add, AddAssign, BitOrAssign, Mul, MulAssign, Rem, Shl, ShlAssign,
        Shr, ShrAssign, Sub, SubAssign,
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

// TODO: remove this trait when [feature(bigint_helper_methods)] got stable.
pub(crate) trait BigIntHelper: Sized {
    /// `self + rhs + carry` (full adder), along with a boolean indicating
    /// whether an arithmetic overflow occurred.
    fn bih_carrying_add(self, rhs: Self, carry: bool) -> (Self, bool);

    /// `self - rhs - borrow` (full subtractor), along with a boolean
    /// indicating whether an arithmetic overflow occurred.
    fn bih_borrowing_sub(self, rhs: Self, borrow: bool) -> (Self, bool);

    /// `self * rhs + carry` (multiply-accumulate)
    fn bih_carrying_mul(self, rhs: Self, carry: Self) -> (Self, Self);
}

impl BigIntHelper for u128 {
    fn bih_carrying_add(self, rhs: Self, carry: bool) -> (Self, bool) {
        let (t, o1) = self.overflowing_add(rhs);
        let (t, o2) = t.overflowing_add(carry as Self);
        (t, o1 || o2)
    }

    fn bih_borrowing_sub(self, rhs: Self, borrow: bool) -> (Self, bool) {
        let (t, o1) = self.overflowing_sub(rhs);
        let (t, o2) = t.overflowing_sub(borrow as Self);
        (t, o1 || o2)
    }

    fn bih_carrying_mul(self, rhs: Self, carry: Self) -> (Self, Self) {
        let (rl, mut rh) = self.widening_mul(rhs);
        let (rl, ovfl) = rl.overflowing_add(carry);
        rh += ovfl as u128;
        (rl, rh)
    }
}

// [feature(bigint_helper_methods)] does not (yet?) provide an impl of
// u128::widening_mul.
pub(crate) trait WideningMul: Sized {
    /// `self * rhs` (wide multiplication)
    fn widening_mul(self, rhs: Self) -> (Self, Self);
}

impl WideningMul for u128 {
    fn widening_mul(self, rhs: Self) -> (Self, Self) {
        let xh = u128_hi(self);
        let xl = u128_lo(self);
        let yh = u128_hi(rhs);
        let yl = u128_lo(rhs);
        let mut rl = xl * yl;
        let t1 = xl * yh;
        let t2 = xh * yl;
        let mut rh = xh * yh;
        let (t1, mut carry) = t1.overflowing_add(t2);
        rh += ((carry as Self) << 64) + u128_hi(t1);
        (rl, carry) = rl.overflowing_add(u128_lo(t1) << 64);
        rh += carry as Self;
        (rl, rh)
    }
}

pub(crate) trait BigUIntHelper: Sized {
    type Output;

    fn widening_shl(self, shift: u32) -> Self::Output;
    fn carrying_shl(self, shift: u32, carry: Self) -> Self::Output;
    fn widening_shr(self, shift: u32) -> Self::Output;
    fn carrying_shr(self, shift: u32, carry: Self) -> Self::Output;
}

impl BigUIntHelper for u128 {
    type Output = (Self, Self);

    #[inline]
    fn widening_shl(self, shift: u32) -> (Self, Self) {
        debug_assert!(shift < Self::BITS);
        (self << shift, self >> (Self::BITS - shift))
    }

    #[inline]
    fn carrying_shl(self, shift: u32, carry: Self) -> Self::Output {
        debug_assert!(shift < Self::BITS);
        debug_assert!(carry < (1_u128 << shift));
        ((self << shift) | carry, self >> (Self::BITS - shift))
    }

    #[inline]
    fn widening_shr(self, shift: u32) -> (Self, Self) {
        debug_assert!(shift < Self::BITS);
        (self >> shift, self << (Self::BITS - shift))
    }

    #[inline]
    fn carrying_shr(self, shift: u32, carry: Self) -> Self::Output {
        debug_assert!(shift < Self::BITS);
        debug_assert!((carry << shift) == 0);
        (carry | (self >> shift), self << (Self::BITS - shift))
    }
}

// Calculate ⌊(x * y) / 2⁵¹²⌋.
pub(crate) fn u256_truncating_mul_u512(x: &u256, y: &u512) -> u256 {
    let mut carry = 0_u128;
    let mut l = (0_u128, 0_u128, 0_u128, 0_u128);
    let mut h = (0_u128, 0_u128, 0_u128, 0_u128);
    (_, carry) = y.lo.lo.widening_mul(x.lo);
    (l.0, carry) = y.lo.hi.bih_carrying_mul(x.lo, carry);
    (l.1, carry) = y.hi.lo.bih_carrying_mul(x.lo, carry);
    (l.2, carry) = y.hi.hi.bih_carrying_mul(x.lo, carry);
    l.3 = carry;
    (h.0, carry) = y.lo.lo.widening_mul(x.hi);
    (h.1, carry) = y.lo.hi.bih_carrying_mul(x.hi, carry);
    (h.2, carry) = y.hi.lo.bih_carrying_mul(x.hi, carry);
    (h.3, carry) = y.hi.hi.bih_carrying_mul(x.hi, carry);
    let mut hi = carry;
    let (_, carry) = l.0.overflowing_add(h.0);
    let (_, carry) = l.1.bih_carrying_add(h.1, carry);
    let (_, carry) = l.2.bih_carrying_add(h.2, carry);
    let (lo, carry) = l.3.bih_carrying_add(h.3, carry);
    hi += carry as u128;
    u256::new(hi, lo)
}

pub(crate) trait DivRem<RHS = Self> {
    type Output;
    fn div_rem(self, rhs: RHS) -> Self::Output;
}

impl DivRem for u128 {
    type Output = (Self, Self);

    #[inline(always)]
    #[allow(clippy::integer_division)]
    fn div_rem(self, rhs: Self) -> Self::Output {
        (self / rhs, self % rhs)
    }
}

/// Helper type representing unsigned integers of 256 bits.
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Default, Eq, Ord, PartialOrd, PartialEq)]
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

    /// Return true, if `self` is odd.
    #[inline]
    pub(crate) const fn is_odd(&self) -> bool {
        (self.lo & 1_u128) == 1
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
    pub(crate) const fn msb(&self) -> u32 {
        debug_assert!(!self.is_zero());
        Self::BITS - self.leading_zeros() - 1
    }

    /// Add 1 to `self` inplace, wrapping around at Self::MAX.
    #[inline]
    pub(crate) fn incr(&mut self) {
        self.lo = self.lo.wrapping_add(1_u128);
        self.hi = self.hi.wrapping_add((self.lo == 0) as u128);
    }

    /// Subtract 1 from `self` inplace, wrapping around at Self::ZERO.
    #[inline]
    pub(crate) fn decr(&mut self) {
        self.hi = self.hi.wrapping_sub((self.lo == 0) as u128);
        self.lo = self.lo.wrapping_sub(1);
    }

    /// Multiply by 10 and add decimal digit (inplace).
    pub(crate) fn imul10_add(&mut self, d: u8) {
        debug_assert!(
            *self
                <= Self::new(
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

    /// Divide `self` by 2ⁿ and round (tie to even).
    pub(crate) fn div_pow2(&self, mut n: u32) -> Self {
        const TIE: u256 = u256::new(1_u128 << 127, 0);
        let (mut quot, rem) = self.widening_shr(n);
        if rem > TIE || (rem == TIE && (quot.lo & 1) == 1) {
            quot.incr();
        }
        quot
    }

    /// Divide `self` inplace by 2ⁿ and round (tie to even).
    #[inline(always)]
    pub(crate) fn idiv_pow2(&mut self, mut n: u32) {
        *self = self.div_pow2(n);
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

        let (mut q1, mut rhat) = x32.div_rem(y1);
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
        let t = (x32.wrapping_shl(64) + x1).wrapping_sub(q1.wrapping_mul(y));
        let (mut q0, mut rhat) = t.div_rem(y1);
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
        let r = ((t.wrapping_shl(64) + x0).wrapping_sub(q0.wrapping_mul(y)))
            >> shift;
        (Self::new(0_u128, q), r)
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
    #[allow(clippy::integer_division)]
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
            let n_full_chunks = (n - 1) / CHUNK_SIZE;
            let mut all_chunks_zero = true;
            for _ in 0..n_full_chunks {
                (q, r) = q.div_rem(CHUNK_BASE);
                all_chunks_zero = all_chunks_zero && r == 0;
            }
            // last chunk
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
    pub(crate) const fn rem_pow2(&self, n: u32) -> Self {
        match n {
            0 => Self::ZERO,
            1..=127 => Self::new(0, self.lo & ((1 << n) - 1)),
            128..=255 => Self::new(self.hi & ((1 << (n - 128)) - 1), self.lo),
            _ => *self,
        }
    }

    // TODO: remove this function and replace calls to it by op <<
    // when trait fns can be declared const.
    pub(crate) const fn shift_left(&self, rhs: u32) -> Self {
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
            0 => *self,
            _ => unreachable!(),
        }
    }

    /// Compute (self * 2ⁿ) % other.
    pub(crate) fn lshift_rem(&self, other: &u256, n: u32) -> u256 {
        let sh = n % u256::BITS;
        let mut t = u512::new(u256::ZERO, *self);
        t <<= sh;
        let mut r = &t % other;
        for _ in 0..n >> 8 {
            t = u512::new(r, u256::ZERO);
            r = &t % other;
            if r.is_zero() {
                break;
            }
        }
        r
    }

    #[cfg(target_endian = "big")]
    #[allow(clippy::cast_possible_truncation)]
    #[inline]
    /// Raw transmutation to `[u64; 4]` (in native endian order).
    pub(crate) const fn to_bits(self) -> [u64; 4] {
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

    /// `self + rhs`, along with a boolean indicating whether an arithmetic
    /// overflow occurred.
    pub(crate) fn overflowing_add(&self, rhs: &Self) -> (Self, bool) {
        let (lo, carry) = self.lo.overflowing_add(rhs.lo);
        // TODO: change when [feature(bigint_helper_methods)] got stable
        let (hi, carry) = self.hi.bih_carrying_add(rhs.hi, carry);
        (Self { hi, lo }, carry)
    }

    /// `self + rhs + carry` (full adder), along with a boolean indicating
    /// whether an arithmetic overflow occurred.
    pub(crate) fn carrying_add(
        &self,
        rhs: &Self,
        carry: bool,
    ) -> (Self, bool) {
        let (mut t, o1) = self.overflowing_add(rhs);
        let mut o2 = false;
        (t.lo, o2) = t.lo.overflowing_add(carry as u128);
        (t.hi, o2) = t.hi.overflowing_add(o2 as u128);
        (t, o1 || o2)
    }

    /// `self - rhs`, along with a boolean indicating whether an arithmetic
    /// overflow occurred.
    pub(crate) fn overflowing_sub(&self, rhs: &Self) -> (Self, bool) {
        let (lo, borrow) = self.lo.overflowing_sub(rhs.lo);
        // TODO: change when [feature(bigint_helper_methods)] got stable
        let (hi, borrow) = self.hi.bih_borrowing_sub(rhs.hi, borrow);
        (Self { hi, lo }, borrow)
    }

    /// Calculate (x - y) % 2²⁵⁶.
    fn wrapping_sub(&self, rhs: &Self) -> Self {
        self.overflowing_sub(rhs).0
    }

    /// `self - rhs - borrow` (full subtractor), along with a boolean
    /// indicating whether an arithmetic overflow occurred.
    pub(crate) fn borrowing_sub(
        self,
        rhs: &Self,
        borrow: bool,
    ) -> (Self, bool) {
        let (mut t, o1) = self.overflowing_sub(rhs);
        let mut o2 = false;
        (t.lo, o2) = t.lo.overflowing_sub(borrow as u128);
        (t.hi, o2) = t.hi.overflowing_sub(o2 as u128);
        (t, o1 || o2)
    }

    /// `self * rhs` (wide multiplication).
    pub(crate) fn widening_mul(&self, rhs: &Self) -> (Self, Self) {
        let (ll, carry) = self.lo.widening_mul(rhs.lo);
        let (lh, hl) = self.lo.bih_carrying_mul(rhs.hi, carry);
        // TODO: change when [feature(bigint_helper_methods)] got stable
        let (lh, carry) = self.hi.bih_carrying_mul(rhs.lo, lh);
        let (hl, incr) = hl.overflowing_add(carry);
        let (hl, mut hh) = self.hi.bih_carrying_mul(rhs.hi, hl);
        hh += incr as u128;
        let hi = u256::new(hh, hl);
        let lo = u256::new(lh, ll);
        (lo, hi)
    }

    /// `self * rhs + carry` (multiply-accumulate)
    fn carrying_mul(&self, rhs: &Self, carry: &Self) -> (Self, Self) {
        let (rl, mut rh) = self.widening_mul(rhs);
        let (rl, ovfl) = rl.overflowing_add(carry);
        rh += ovfl as u128;
        (rl, rh)
    }

    /// Calculate ⌊(self * rhs) / 2²⁵⁶⌋.
    pub(crate) fn truncating_mul(&self, rhs: &Self) -> Self {
        self.widening_mul(rhs).1
    }

    /// Calculate (self * rhs) % 2²⁵⁶.
    pub(crate) fn wrapping_mul(&self, rhs: &Self) -> Self {
        let (rl, mut rh) = self.lo.widening_mul(rhs.lo);
        rh = rh.wrapping_add(self.lo.wrapping_mul(rhs.hi));
        rh = rh.wrapping_add(self.hi.wrapping_mul(rhs.lo));
        Self::new(rh, rl)
    }
}

impl fmt::Debug for u256 {
    fn fmt(&self, form: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(form, "(0x{:032x}, 0x{:032x})", self.hi, self.lo)
    }
}

impl Add for &u256 {
    type Output = u256;

    fn add(self, rhs: Self) -> Self::Output {
        let (r, carry) = self.overflowing_add(rhs);
        assert!(!carry, "Attempt to add with overflow");
        r
    }
}

impl Add<u32> for &u256 {
    type Output = u256;

    #[inline(always)]
    fn add(self, rhs: u32) -> Self::Output {
        self + rhs as u128
    }
}

impl Add<u128> for &u256 {
    type Output = u256;

    fn add(self, rhs: u128) -> Self::Output {
        let (lo, carry) = self.lo.overflowing_add(rhs);
        let (hi, carry) = self.hi.overflowing_add(carry as u128);
        assert!(!carry, "Attempt to add with overflow");
        Self::Output { hi, lo }
    }
}

impl AddAssign<&Self> for u256 {
    fn add_assign(&mut self, rhs: &Self) {
        let mut carry = false;
        (self.lo, carry) = self.lo.overflowing_add(rhs.lo);
        // TODO: change when [feature(bigint_helper_methods)] got stable
        (self.hi, carry) = self.hi.bih_carrying_add(rhs.hi, carry);
        assert!(!carry, "Attempt to add with overflow");
    }
}

impl AddAssign<u128> for u256 {
    fn add_assign(&mut self, rhs: u128) {
        let mut carry = false;
        (self.lo, carry) = self.lo.overflowing_add(rhs);
        (self.hi, carry) = self.hi.overflowing_add(carry as u128);
        assert!(!carry, "Attempt to add with overflow");
    }
}

impl Sub for &u256 {
    type Output = u256;

    fn sub(self, rhs: Self) -> Self::Output {
        let (r, borrow) = self.overflowing_sub(rhs);
        assert!(!borrow, "Attempt to subtract with overflow");
        r
    }
}

impl Sub<u32> for &u256 {
    type Output = u256;

    #[inline(always)]
    fn sub(self, rhs: u32) -> Self::Output {
        self - rhs as u128
    }
}

impl Sub<u128> for &u256 {
    type Output = u256;

    fn sub(self, rhs: u128) -> Self::Output {
        let (lo, borrow) = self.lo.overflowing_sub(rhs);
        let (hi, borrow) = self.hi.overflowing_sub(borrow as u128);
        assert!(!borrow, "Attempt to subtract with overflow");
        Self::Output { hi, lo }
    }
}

impl SubAssign<&Self> for u256 {
    fn sub_assign(&mut self, rhs: &Self) {
        let mut borrow = false;
        (self.lo, borrow) = self.lo.overflowing_sub(rhs.lo);
        // TODO: change when [feature(bigint_helper_methods)] got stable
        (self.hi, borrow) = self.hi.bih_borrowing_sub(rhs.hi, borrow);
        assert!(!borrow, "Attempt to subtract with overflow");
    }
}

impl SubAssign<u128> for u256 {
    fn sub_assign(&mut self, rhs: u128) {
        let mut borrow = false;
        (self.lo, borrow) = self.lo.overflowing_sub(rhs);
        (self.hi, borrow) = self.hi.overflowing_sub(borrow as u128);
        assert!(!borrow, "Attempt to subtract with overflow");
    }
}

impl Mul<&u256> for &u256 {
    type Output = u256;

    fn mul(self, rhs: &u256) -> Self::Output {
        assert!(
            self.hi == 0 || rhs.hi == 0,
            "Attempt to multiply with overflow."
        );
        let (lo, mut hi) = self.lo.widening_mul(rhs.lo);
        let (mut t, mut ovfl) = self.lo.overflowing_mul(rhs.hi);
        assert!(!ovfl, "Attempt to multiply with overflow.");
        (hi, ovfl) = hi.overflowing_add(t);
        assert!(!ovfl, "Attempt to multiply with overflow.");
        (t, ovfl) = self.hi.overflowing_mul(rhs.lo);
        assert!(!ovfl, "Attempt to multiply with overflow.");
        (hi, ovfl) = hi.overflowing_add(t);
        assert!(!ovfl, "Attempt to multiply with overflow.");
        Self::Output::new(hi, lo)
    }
}

impl MulAssign<u128> for u256 {
    fn mul_assign(&mut self, rhs: u128) {
        let (t, carry) = self.lo.widening_mul(rhs);
        self.lo = t;
        let (t, mut ovfl) = self.hi.overflowing_mul(rhs);
        assert!(!ovfl, "Attempt to multiply with overflow.");
        (self.hi, ovfl) = t.overflowing_add(carry);
        assert!(!ovfl, "Attempt to multiply with overflow.");
    }
}

impl DivRem<u64> for &u256 {
    type Output = (u256, u64);

    /// Returns `self` / rhs, `self` % rhs
    #[allow(clippy::cast_possible_truncation)]
    fn div_rem(self, rhs: u64) -> Self::Output {
        let (quot_hi, r) = self.hi.div_rem(rhs as u128);
        let (mut quot_lo, r) =
            ((r << 64) + u128_hi(self.lo)).div_rem(rhs as u128);
        quot_lo <<= 64;
        let (t, r) = ((r << 64) + u128_lo(self.lo)).div_rem(rhs as u128);
        quot_lo += t;
        (u256::new(quot_hi, quot_lo), r as u64)
    }
}

impl DivRem<u128> for &u256 {
    type Output = (u256, u128);

    /// Returns `self` / rhs, `self` % rhs
    #[allow(clippy::integer_division)]
    #[allow(clippy::cast_possible_truncation)]
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
    #[allow(clippy::integer_division)]
    fn div_rem(self, rhs: &u256) -> Self::Output {
        debug_assert!(!rhs.is_zero());
        if rhs.hi == 0 {
            let (quot, rem) = self.div_rem(rhs.lo);
            (quot, u256::new(0, rem))
        } else if rhs.hi > self.hi {
            // self < rhs
            return (u256::ZERO, *self);
        } else {
            // estimate the quotient
            let nlz = self.hi.leading_zeros();
            let mut quot = (self << nlz).hi / (rhs << nlz).hi;
            // trim the estimate
            let mut t = *rhs;
            t *= quot;
            if t > *self {
                let mut d = &t - self;
                let (mut n, _) = d.div_rem(rhs);
                n.incr();
                debug_assert_eq!(n.hi, 0);
                quot -= n.lo;
                d = *rhs;
                d *= n.lo;
                t -= &d;
            } else {
                let u = &t + rhs;
                if u < *self {
                    let mut d = self - &t;
                    let (n, _) = d.div_rem(rhs);
                    debug_assert_eq!(n.hi, 0);
                    quot += n.lo;
                    d = *rhs;
                    d *= n.lo;
                    t += &d;
                }
            }
            let rem = self - &t;
            (u256::new(0, quot), rem)
        }
    }
}

impl Rem<u64> for &u256 {
    type Output = u64;

    #[inline]
    #[allow(clippy::cast_possible_truncation)]
    fn rem(self, rhs: u64) -> Self::Output {
        (self % rhs as u128) as u64
    }
}

impl Rem<u128> for &u256 {
    type Output = u128;

    #[inline]
    fn rem(self, rhs: u128) -> Self::Output {
        let t = u256::new(self.hi % rhs, self.lo);
        let (_, rem) = t.div_rem_u128_special(rhs);
        rem
    }
}

impl Rem<&u256> for &u256 {
    type Output = u256;

    #[inline]
    fn rem(self, rhs: &u256) -> Self::Output {
        let (_, rem) = self.div_rem(rhs);
        rem
    }
}

impl BitOrAssign for u256 {
    fn bitor_assign(&mut self, rhs: Self) {
        self.hi |= rhs.hi;
        self.lo |= rhs.lo;
    }
}

impl BigUIntHelper for &u256 {
    type Output = (u256, u256);

    fn widening_shl(self, mut shift: u32) -> Self::Output {
        debug_assert!(shift < u256::BITS);
        match shift {
            1..=127 => {
                let (lo, carry) = self.lo.widening_shl(shift);
                let (hi, carry) = self.hi.carrying_shl(shift, carry);
                (u256::new(hi, lo), u256::new(0_u128, carry))
            }
            128 => (u256::new(self.lo, 0_u128), u256::new(0_u128, self.hi)),
            129..=255 => {
                shift -= 128;
                let (lo, carry) = self.lo.widening_shl(shift);
                let (hi, carry) = self.hi.carrying_shl(shift, carry);
                (u256::new(lo, 0_u128), u256::new(carry, hi))
            }
            0 => (*self, u256::ZERO),
            _ => unreachable!(),
        }
    }

    fn carrying_shl(self, shift: u32, carry: Self) -> Self::Output {
        debug_assert!(shift < u256::BITS);
        let (mut shifted, c) = self.widening_shl(shift);
        shifted |= *carry;
        (shifted, c)
    }

    fn widening_shr(self, mut shift: u32) -> Self::Output {
        debug_assert!(shift < 2 * u256::BITS);
        match shift {
            1..=127 => {
                let (hi, carry) = self.hi.widening_shr(shift);
                let (lo, carry) = self.lo.carrying_shr(shift, carry);
                (u256::new(hi, lo), u256::new(carry, 0_u128))
            }
            128 => (u256::new(0_u128, self.hi), u256::new(self.lo, 0_u128)),
            129..=255 => {
                shift -= 128;
                let (hi, carry) = self.hi.widening_shr(shift);
                let (lo, carry) = self.lo.carrying_shr(shift, carry);
                (u256::new(0_u128, hi), u256::new(lo, carry))
            }
            256 => (u256::ZERO, *self),
            257..=511 => (u256::ZERO, self.widening_shr(shift - 256).0),
            0 => (*self, u256::ZERO),
            _ => unreachable!(),
        }
    }

    fn carrying_shr(self, shift: u32, carry: Self) -> Self::Output {
        debug_assert!(shift < u256::BITS);
        let (mut shifted, c) = self.widening_shr(shift);
        shifted |= *carry;
        (shifted, c)
    }
}

impl Shl<u32> for &u256 {
    type Output = u256;

    fn shl(self, rhs: u32) -> Self::Output {
        assert!(
            rhs < Self::Output::BITS,
            "Attempt to shift left with overflow."
        );
        self.widening_shl(rhs).0
    }
}

impl ShlAssign<u32> for u256 {
    fn shl_assign(&mut self, rhs: u32) {
        assert!(rhs < Self::BITS, "Attempt to shift left with overflow.");
        *self = self.widening_shl(rhs).0;
    }
}

impl Shr<u32> for &u256 {
    type Output = u256;

    fn shr(self, rhs: u32) -> Self::Output {
        assert!(
            rhs <= 2 * Self::Output::BITS,
            "Attempt to shift right with underflow."
        );
        self.widening_shr(rhs).0
    }
}

impl ShrAssign<u32> for u256 {
    fn shr_assign(&mut self, rhs: u32) {
        assert!(
            rhs <= (Self::BITS - 1),
            "Attempt to shift right with underflow."
        );
        *self = self.widening_shr(rhs).0;
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
#[derive(Clone, Copy, Default, Eq, Ord, PartialOrd, PartialEq)]
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

    /// Create an `u512` value from two u256 values.
    #[inline(always)]
    pub(crate) const fn new(hi: u256, lo: u256) -> Self {
        Self { hi, lo }
    }

    /// Create an `u512` value from a tuple of two u256 values (big endian).
    #[inline(always)]
    pub(crate) const fn from_big_endian_tuple(t: (u256, u256)) -> Self {
        Self { hi: t.0, lo: t.1 }
    }

    /// Create an `u512` value from a tuple of two u256 values (little
    /// endian).
    #[inline(always)]
    pub(crate) const fn from_little_endian_tuple(t: (u256, u256)) -> Self {
        Self { hi: t.1, lo: t.0 }
    }

    /// Return true, if `self` == 0.
    #[inline]
    pub(crate) const fn is_zero(&self) -> bool {
        self.hi.is_zero() && self.lo.is_zero()
    }

    /// Returns the number of leading zeros in the binary representation of
    /// `self`.
    pub(crate) const fn leading_zeros(&self) -> u32 {
        self.hi.leading_zeros()
            + (self.hi.is_zero()) as u32 * self.lo.leading_zeros()
    }

    /// Returns the index of the most significant bit of `self`.
    /// Pre-condition: `self` must not be zero!
    pub(crate) const fn msb(&self) -> u32 {
        debug_assert!(!self.is_zero());
        Self::BITS - self.leading_zeros() - 1
    }

    /// Add 1 to `self` inplace, wrapping around at Self::MAX.
    #[inline]
    pub(crate) fn incr(&mut self) {
        self.lo.incr();
        if self.lo.is_zero() {
            self.hi.incr();
        }
    }

    /// Subtract 1 from `self` inplace, wrapping around at Self::ZERO.
    #[inline]
    pub(crate) fn decr(&mut self) {
        if self.lo.is_zero() {
            self.hi.decr();
        }
        self.lo.decr();
    }

    /// `self * rhs` (wide multiplication).
    pub(crate) fn widening_mul(&self, rhs: &Self) -> (Self, Self) {
        let (ll, carry) = self.lo.widening_mul(&rhs.lo);
        let (lh, hl) = self.lo.carrying_mul(&rhs.hi, &carry);
        let (lh, carry) = self.hi.carrying_mul(&rhs.lo, &lh);
        let (hl, incr) = hl.overflowing_add(&carry);
        let (hl, mut hh) = self.hi.carrying_mul(&rhs.hi, &hl);
        hh += incr as u128;
        let hi = u512::new(hh, hl);
        let lo = u512::new(lh, ll);
        (lo, hi)
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
    //noinspection DuplicatedCode
    pub(crate) fn div_rem_u256_special(&self, rhs: &u256) -> (Self, u256) {
        debug_assert!(self.hi < *rhs);
        const B: u256 = u256::new(1, 0);
        // Normalize dividend and divisor, so that the divisor has its highest
        // bit set, and get their 128-bit parts.
        let shift = rhs.leading_zeros();
        let mut x = *self;
        x <<= shift;
        let x32 = x.hi;
        let x1 = x.lo.hi;
        let x0 = x.lo.lo;
        let y = rhs << shift;
        let y1 = u256::new(0, y.hi);
        let y0 = u256::new(0, y.lo);

        let (mut q1, mut rhat) = x32.div_rem(&y1);
        // Now we have
        // q1 * y1 + rhat = x32
        // so that
        // q1 * y1 * 2¹²⁸ + rhat * 2¹²⁸ + x1 = x32 * 2¹²⁸ + x1
        while q1 >= B || &q1 * &y0 > &(&rhat * &B) + x1 {
            q1.decr();
            rhat += &y1;
            if rhat >= B {
                break;
            }
        }
        // The loop did not change the equation given above. It was terminated
        // if either q1 < 2¹²⁸ or rhat >= 2¹²⁸ or q1 * yn0 > rhat * 2¹²⁸ + x1.
        // In these cases follows:
        // q1 * y0 <= rhat * 2¹²⁸ + x1, therefor
        // q1 * y1 * 2¹²⁸ + q1 * y0 <= x32 * 2¹²⁸ + x1, and
        // q1 * y <= x32 * 2¹²⁸ + x1, and
        // x32 * 2¹²⁸ + x1 - q1 * y >= 0.
        // That means that the add-back step in Knuth's algorithm D is not
        // required.

        // Since the final quotient is < 2²⁵⁶, this must also be true for
        // x32 * 2¹²⁸ + x1 - q1 * y. Thus, in the following we can safely
        // ignore any possible overflow in x32 * 2¹²⁸ or q1 * y.
        let mut t = u256::new(x32.lo, x1);
        t = t.wrapping_sub(&q1.wrapping_mul(&y));
        let (mut q0, mut rhat) = t.div_rem(&y1);
        while q0 >= B || &q0 * &y0 > &(&rhat * &B) + x0 {
            q0.decr();
            rhat += &y1;
            if rhat >= B {
                break;
            }
        }
        // q = q1 * B + q0
        let mut q = u256::new(q1.lo, 0);
        q += &q0;
        // Denormalize remainder
        let mut r = u256::new(t.lo, x0);
        r = r.wrapping_sub(&q0.wrapping_mul(&y));
        r >>= shift;
        (Self::new(u256::ZERO, q), r)
    }

    /// Divide `self` inplace by 2ⁿ and round (tie to even).
    pub(crate) fn idiv_pow2(&mut self, mut n: u32) {
        const TIE: u512 =
            u512::new(u256::new(1_u128 << 127, 0_u128), u256::ZERO);
        let (quot, rem) = self.widening_shr(n);
        *self = quot;
        if rem > TIE || (rem == TIE && (self.lo.lo & 1) == 1) {
            self.incr();
        }
    }

    /// Returns `self` / 10ⁿ, rounded tie to even.
    #[allow(clippy::integer_division)]
    pub(crate) fn div_pow10_rounded(&self, n: u32) -> Self {
        const CHUNK_SIZE: u32 = 38;
        const CHUNK_BASE: u128 = 10_u128.pow(CHUNK_SIZE);
        debug_assert_ne!(n, 0);
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
            let n_chunks = (n - 1) / CHUNK_SIZE;
            let mut all_chunks_zero = true;
            for _ in 0..n_chunks {
                (q, r) = q.div_rem(CHUNK_BASE);
                all_chunks_zero = all_chunks_zero && r == 0;
            }
            let d = 10_u128.pow(n - n_chunks * CHUNK_SIZE);
            (q, r) = q.div_rem(d);
            let tie = d >> 1;
            if r > tie
                || (r == tie && ((q.lo.lo & 1) == 1 || !all_chunks_zero))
            {
                q.incr();
            }
        }
        q
    }
}

impl fmt::Debug for u512 {
    fn fmt(&self, form: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            form,
            "(0x{:032x}, 0x{:032x}, 0x{:032x}, 0x{:032x})",
            self.hi.hi, self.hi.lo, self.lo.hi, self.lo.lo,
        )
    }
}

impl AddAssign<&Self> for u512 {
    fn add_assign(&mut self, rhs: &Self) {
        let mut carry = false;
        (self.lo, carry) = self.lo.overflowing_add(&rhs.lo);
        (self.hi, carry) = self.hi.carrying_add(&rhs.hi, carry);
        assert!(!carry, "Attempt to add with overflow");
    }
}

impl SubAssign<&Self> for u512 {
    fn sub_assign(&mut self, rhs: &Self) {
        let mut borrow = false;
        (self.lo, borrow) = self.lo.overflowing_sub(&rhs.lo);
        (self.hi, borrow) = self.hi.borrowing_sub(&rhs.hi, borrow);
        assert!(!borrow, "Attempt to add with overflow");
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

impl DivRem<&u256> for &u512 {
    type Output = (u512, u256);

    /// Returns `self` / rhs, `self` % rhs
    fn div_rem(self, rhs: &u256) -> Self::Output {
        if self.hi.is_zero() {
            let (quot, rem) = self.lo.div_rem(rhs);
            (u512::new(u256::ZERO, quot), rem)
        } else if rhs.hi == 0 {
            let (quot, rem) = self.div_rem(rhs.lo);
            (quot, u256::new(0, rem))
        } else if self.hi < *rhs {
            self.div_rem_u256_special(rhs)
        } else {
            let mut quot = *self;
            let mut rem = u256::ZERO;
            quot.hi = &quot.hi % rhs;
            (quot, rem) = quot.div_rem_u256_special(rhs);
            (quot.hi, _) = self.hi.div_rem(rhs);
            (quot, rem)
        }
    }
}

impl Rem<u64> for &u512 {
    type Output = u64;

    #[inline(always)]
    #[allow(clippy::cast_possible_truncation)]
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

impl Rem<&u256> for &u512 {
    type Output = u256;

    #[inline]
    fn rem(self, rhs: &u256) -> Self::Output {
        let t = u512::new(&self.hi % rhs, self.lo);
        let (_, rem) = t.div_rem_u256_special(rhs);
        rem
    }
}

impl BitOrAssign for u512 {
    fn bitor_assign(&mut self, rhs: Self) {
        self.hi |= rhs.hi;
        self.lo |= rhs.lo;
    }
}

impl BigUIntHelper for &u512 {
    type Output = (u512, u512);

    fn widening_shl(self, mut shift: u32) -> Self::Output {
        debug_assert!(shift < u512::BITS);
        match shift {
            1..=255 => {
                let (lo, carry) = self.lo.widening_shl(shift);
                let (hi, carry) = self.hi.carrying_shl(shift, &carry);
                (u512::new(hi, lo), u512::new(u256::ZERO, carry))
            }
            256 => (
                u512::new(self.lo, u256::ZERO),
                u512::new(u256::ZERO, self.hi),
            ),
            257..=511 => {
                shift -= 256;
                let (lo, carry) = self.lo.widening_shl(shift);
                let (hi, carry) = self.hi.carrying_shl(shift, &carry);
                (u512::new(lo, u256::ZERO), u512::new(carry, hi))
            }
            0 => (*self, u512::ZERO),
            _ => unreachable!(),
        }
    }

    fn carrying_shl(self, shift: u32, carry: Self) -> Self::Output {
        debug_assert!(shift < u512::BITS);
        let (mut shifted, c) = self.widening_shl(shift);
        shifted |= *carry;
        (shifted, c)
    }

    fn widening_shr(self, mut shift: u32) -> Self::Output {
        debug_assert!(shift < u512::BITS);
        match shift {
            1..=255 => {
                let (hi, carry) = self.hi.widening_shr(shift);
                let (lo, carry) = self.lo.carrying_shr(shift, &carry);
                (u512::new(hi, lo), u512::new(carry, u256::ZERO))
            }
            256 => (
                u512::new(u256::ZERO, self.hi),
                u512::new(self.lo, u256::ZERO),
            ),
            257..=511 => {
                shift -= 256;
                let (hi, carry) = self.hi.widening_shr(shift);
                let (lo, carry) = self.lo.carrying_shr(shift, &carry);
                (u512::new(u256::ZERO, hi), u512::new(lo, carry))
            }
            0 => (*self, u512::ZERO),
            _ => unreachable!(),
        }
    }

    fn carrying_shr(self, shift: u32, carry: Self) -> Self::Output {
        debug_assert!(shift < u512::BITS);
        let (mut shifted, c) = self.widening_shr(shift);
        shifted |= *carry;
        (shifted, c)
    }
}

impl Shl<u32> for &u512 {
    type Output = u512;

    fn shl(self, rhs: u32) -> Self::Output {
        assert!(
            rhs < Self::Output::BITS,
            "Attempt to shift left with overflow."
        );
        self.widening_shl(rhs).0
    }
}

impl ShlAssign<u32> for u512 {
    fn shl_assign(&mut self, rhs: u32) {
        assert!(rhs < Self::BITS, "Attempt to shift left with overflow.");
        *self = self.widening_shl(rhs).0;
    }
}

impl Shr<u32> for &u512 {
    type Output = u512;

    fn shr(self, rhs: u32) -> Self::Output {
        assert!(
            rhs <= Self::Output::BITS,
            "Attempt to shift right with underflow."
        );
        self.widening_shr(rhs).0
    }
}

impl ShrAssign<u32> for u512 {
    fn shr_assign(&mut self, rhs: u32) {
        assert!(
            rhs <= (Self::BITS - 1),
            "Attempt to shift right with underflow."
        );
        *self = self.widening_shr(rhs).0;
    }
}

impl fmt::Display for u512 {
    #[allow(clippy::cast_possible_truncation)]
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
    fn test_div_rem_1() {
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
                u256::new(
                    34028236692,
                    31934256858593286117999845820724523012
                ),
                564039457584007913129639935
            )
        );
        let v = u256::new(70299, 93425685859328611799984582072);
        assert_eq!(
            v.div_rem(10_u128.pow(27) + 3),
            (u256::new(0, 23921510112175146), 468697630784693143145201978)
        );
    }

    #[test]
    fn test_div_rem_2() {
        let num = u256::new(
            396091524468374439553466833932038,
            287120322474436508255079181112596091133,
        );
        let den = 261829273180548883101888490383232063939_u128;
        let (quot, rem) = num.div_rem(den);
        assert_eq!(quot, u256::new(0, 514774226067836787805468557499791));
        assert_eq!(rem, 170749921628011334804182094129221981712_u128);
        assert_eq!(rem, &num % den);
    }

    #[test]
    fn test_div_rem_3() {
        let num = u256::new(
            339760931524468374439553466833932000838,
            287120322474436508255079181112596091133,
        );
        let den = u256::new(
            26182927318054888310188849038323206,
            34028236692093846346337460743176821145,
        );
        let (quot, rem) = num.div_rem(&den);
        assert_eq!(quot, u256::new(0, 12976));
        assert_eq!(
            rem,
            u256::new(
                11266645388143726542961712650078485,
                82950902321873430177054416653535172045
            )
        );
        assert_eq!(rem, &num % &den);
    }

    #[test]
    fn test_div_rem_by_10() {
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
    #[allow(clippy::integer_division)]
    #[allow(clippy::cast_possible_truncation)]
    fn test_div_rem_by_pow10() {
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
mod u256_div_pow2_tests {
    use super::*;

    #[test]
    fn test_div_pow2() {
        let u = u256::new(
            0x00001000000000000000000000000003,
            0x00001000000000000000000000000002,
        );
        let v = u.div_pow2(2);
        assert_eq!(v, (&u >> 2));
        let v = u.div_pow2(17);
        assert_eq!(v, (&u >> 17));
        let v = u.div_pow2(129);
        assert_eq!(v, &(&u >> 129) + &u256::ONE);
    }

    #[test]
    fn test_idiv_pow2() {
        let u = u256::new(
            0x00001f6a7a2955385e583ebeff65cc22,
            0x6480ae685c3155a037f22051d5c9f93a,
        );
        let mut v = u.clone();
        v.idiv_pow2(12);
        assert_eq!(v, &(&u >> 12) + &u256::ONE);
        let mut v = u.clone();
        v.idiv_pow2(137);
        assert_eq!(v, (&u >> 137));
    }
}

#[cfg(test)]
mod u256_to_str_tests {
    use alloc::string::ToString;

    use super::*;

    #[test]
    fn test_zero() {
        let v = u256::ZERO;
        assert_eq!(v.to_string(), "0");
    }

    #[test]
    fn test_max() {
        let v = u256::MAX;
        assert_eq!(
            v.to_string(),
            "115792089237316195423570985008687907853269984665640564039457584007\
             913129639935"
        );
    }
}

#[cfg(test)]
mod u128_widening_mul_tests {
    use super::*;

    #[test]
    fn test_max() {
        let x = u128::MAX;
        let z = x.widening_mul(x);
        assert_eq!(z, (1, u128::MAX - 1));
    }
}

#[cfg(test)]
mod u256_widening_mul_tests {
    use super::*;

    #[test]
    fn test_max_half() {
        let x = &u256::MAX >> 1;
        let z = x.widening_mul(&x);
        assert_eq!(z, (u256::ONE, &x >> 1));
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
mod u512_add_assign_tests {
    use super::*;

    #[test]
    fn test_add_assign_1() {
        let two = &u256::ONE + &u256::ONE;
        let mut v = u512::new(u256::ONE, u256::ONE);
        let w = v;
        let z = u512::new(two, two);
        v += &w;
        assert_eq!(v, z);
    }

    #[test]
    fn test_add_assign_2() {
        let mut v = u512::new(u256::ZERO, u256::MAX);
        let w = u512::new(u256::ONE, u256::ONE);
        let z = u512::new(&u256::ONE + &u256::ONE, u256::ZERO);
        v += &w;
        assert_eq!(v, z);
    }

    #[test]
    fn test_add_assign_3() {
        let mut v = u512::new(u256::ZERO, u256::MAX);
        let w = v;
        let z = u512::new(u256::ONE, &u256::MAX - &u256::ONE);
        v += &w;
        assert_eq!(v, z);
    }

    #[test]
    #[should_panic]
    fn test_add_assign_ovfl() {
        let mut v = u512::new(u256::ZERO, u256::MAX);
        let w = u512::new(u256::MAX, u256::ONE);
        v += &w;
    }
}

#[cfg(test)]
mod u512_sub_assign_tests {
    use super::*;

    #[test]
    fn test_sub_assign_1() {
        let two = &u256::ONE + &u256::ONE;
        let mut v = u512::new(two, two);
        let w = u512::new(u256::ONE, u256::ONE);
        v -= &w;
        assert_eq!(v, w);
    }

    #[test]
    fn test_sub_assign_2() {
        let mut v = u512::new(u256::MAX, u256::ZERO);
        let w = u512::new(u256::ZERO, u256::ONE);
        let z = u512::new(&u256::MAX - &u256::ONE, u256::MAX);
        v -= &w;
        assert_eq!(v, z);
    }

    #[test]
    fn test_sub_assign_3() {
        let mut v = u512::new(u256::ONE, u256::MAX);
        let w = u512::new(u256::ONE, &u256::MAX - &u256::ONE);
        let z = u512::new(u256::ZERO, u256::ONE);
        v -= &w;
        assert_eq!(v, z);
    }

    #[test]
    #[should_panic]
    fn test_sub_assign_ovfl() {
        let mut v = u512::new(u256::ONE, &u256::MAX - &u256::ONE);
        let w = u512::new(u256::ONE, u256::MAX);
        v -= &w;
    }
}

#[cfg(test)]
mod u512_div_rem_tests {
    use super::*;

    #[test]
    fn test_div_rem_1() {
        let v = u512::MAX;
        assert_eq!(
            v.div_rem(2_u128),
            (u512::new(&u256::MAX >> 1, u256::MAX), 1)
        );
    }

    #[test]
    fn test_div_rem_2() {
        let num = u512::new(
            u256::new(
                134028236692093846346337460743176821145_u128,
                204169420152563078078024764459060926873_u128,
            ),
            u256::new(
                74093960915244683744395534668339322228_u128,
                287120322474436508255079181112596091133_u128,
            ),
        );
        let den = 261829273180548883101888490383232063939_u128;
        let (quot, rem) = num.div_rem(den);
        assert_eq!(
            quot,
            u512::new(
                u256::new(
                    0_u128,
                    174187725695499550061058271888170382541_u128
                ),
                u256::new(
                    309703370695449177740528536891543921158_u128,
                    180178614806750523772503248486955614504_u128
                )
            )
        );
        assert_eq!(rem, 101817417540912105633242427321208801157_u128);
        assert_eq!(rem, &num % den);
    }

    #[test]
    fn test_div_rem_by_10() {
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
    fn test_div_rem_by_pow10() {
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
