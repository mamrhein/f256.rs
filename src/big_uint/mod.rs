// ---------------------------------------------------------------------------
// Copyright:   (c) 2022 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

mod uint128;
mod uint_generic;

use alloc::vec::Vec;
use core::{
    fmt::{Debug, Display},
    mem::size_of,
    ops::{
        Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, Div, Mul,
        MulAssign, Rem, Shl, ShlAssign, Shr, ShrAssign, Sub, SubAssign,
    },
};

pub(crate) use uint128::U128;
use uint128::{u128_hi, u128_lo};
pub(crate) use uint_generic::{UInt, U1024, U256, U512};

pub(crate) trait HiLo
where
    Self: Copy + Clone + Sized,
{
    type T;
    /// Returns a new instance of Self.
    fn from_hi_lo(hi: Self::T, lo: Self::T) -> Self;
    fn hi_t(&self) -> Self::T;
    fn lo_t(&self) -> Self::T;
    fn hi(&self) -> Self;
    fn lo(&self) -> Self;
    fn as_vec_u128(&self) -> Vec<u128>;
}

pub(crate) trait DivRem<RHS = Self> {
    type Output;
    /// Returns `self` / `rhs`, `self` % `rhs`
    fn div_rem(self, rhs: RHS) -> Self::Output;
}

pub(crate) trait BigUInt
where
    Self: Copy
        + Clone
        + Debug
        + Default
        + Display
        + Sized
        + Add<Output = Self>
        + Add<u128, Output = Self>
        + for<'a> AddAssign<&'a Self>
        + BitAnd<Output = Self>
        + for<'a> BitAndAssign<&'a Self>
        + BitOr<Output = Self>
        + for<'a> BitOrAssign<&'a Self>
        + Div<Output = Self>
        + DivRem<u128, Output = (Self, u128)>
        + DivRem<Output = (Self, Self)>
        + for<'a> From<&'a u128>
        + for<'a> From<&'a [u128]>
        + Mul<Output = Self>
        + for<'a> MulAssign<&'a Self>
        + PartialEq
        + PartialOrd
        + Ord
        + Rem<Output = Self>
        + Shl<u32, Output = Self>
        + ShlAssign<u32>
        + Shr<u32, Output = Self>
        + ShrAssign<u32>
        + Sub<Output = Self>
        + for<'a> SubAssign<&'a Self>,
{
    const BITS: u32 = size_of::<Self>() as u32 * 8;
    const ZERO: Self;
    const ONE: Self;
    const TWO: Self;
    const MAX: Self;

    /// Return true, if `self` == 0.
    #[inline(always)]
    fn is_zero(&self) -> bool {
        *self == Self::ZERO
    }

    /// Return true, if `self` is even.
    fn is_even(&self) -> bool;

    /// Return true, if `self` is odd.
    fn is_odd(&self) -> bool;

    /// Returns the number of leading zeros in the binary representation of
    /// `self`.
    fn leading_zeros(&self) -> u32;

    /// Returns the number of trailing zeros in the binary representation of
    /// `self`.
    fn trailing_zeros(&self) -> u32;

    /// Returns the index of the most significant bit of `self`.
    /// Pre-condition: `self` must not be zero!
    fn msb(&self) -> u32 {
        debug_assert!(!self.is_zero());
        Self::BITS - self.leading_zeros() - 1
    }

    /// Add 1 to `self` inplace, wrapping around at Self::MAX.
    fn incr(&mut self);

    /// If `cond`, add 1 to `self` inplace, wrapping around at Self::MAX.
    #[inline]
    fn incr_if(&mut self, cond: bool) {
        *self = self.wrapping_add(&Self::from(&(cond as u128)));
    }

    /// Subtract 1 from `self` inplace, wrapping around at Self::ZERO.
    fn decr(&mut self);

    /// If `cond` subtract 1 from `self` inplace, wrapping around at
    /// Self::ZERO.
    #[inline]
    fn decr_if(&mut self, cond: bool) {
        *self = self.wrapping_sub(&Self::from(&(cond as u128)));
    }

    /// Returns `self + rhs`, along with a boolean indicating whether an
    /// arithmetic overflow occurred.
    fn overflowing_add(&self, rhs: &Self) -> (Self, bool);

    /// Returns `self + rhs + carry` (full adder), along with a boolean
    /// indicating whether an arithmetic overflow occurred.
    fn carrying_add(&self, rhs: &Self, carry: bool) -> (Self, bool) {
        let (t, o1) = self.overflowing_add(rhs);
        let (t, o2) = t.overflowing_add(&Self::from(&(carry as u128)));
        (t, o1 || o2)
    }

    /// Wrapping (modular) addition.
    /// Returns `self + rhs`, wrapping around at the boundary of the type.
    fn wrapping_add(&self, rhs: &Self) -> Self;

    /// Returns `self - rhs`, along with a boolean indicating whether an
    /// arithmetic overflow occurred.
    fn overflowing_sub(&self, rhs: &Self) -> (Self, bool);

    /// Returns `self - rhs - borrow` (full subtractor), along with a boolean
    /// indicating whether an arithmetic overflow occurred.
    fn borrowing_sub(self, rhs: Self, borrow: bool) -> (Self, bool) {
        let (t, o1) = self.overflowing_sub(&rhs);
        let (t, o2) = t.overflowing_sub(&Self::from(&(borrow as u128)));
        (t, o1 || o2)
    }

    /// Wrapping (modular) subtraction.
    /// Returns `self - rhs`, wrapping around at the boundary of the type.
    fn wrapping_sub(&self, rhs: &Self) -> Self;

    /// Returns `self * rhs`, along with a boolean indicating whether an
    /// arithmetic overflow occurred.
    fn overflowing_mul(&self, rhs: &Self) -> (Self, bool);

    /// Returns `self * rhs` (wide multiplication)
    fn widening_mul(&self, rhs: &Self) -> (Self, Self);

    /// Returns `self * rhs + carry` (multiply-accumulate)
    fn carrying_mul(&self, rhs: &Self, carry: &Self) -> (Self, Self) {
        let (rl, mut rh) = self.widening_mul(rhs);
        let (rl, ovfl) = rl.overflowing_add(carry);
        rh.incr_if(ovfl);
        (rl, rh)
    }

    /// Calculate ⌊(self * rhs) / 2ⁿ⌋ where n = Self::BITS.
    fn truncating_mul(&self, rhs: &Self) -> Self {
        self.widening_mul(rhs).1
    }

    /// Calculate (self * rhs) % 2ⁿ where n = Self::BITS.
    fn wrapping_mul(&self, rhs: &Self) -> Self {
        self.widening_mul(rhs).0
    }

    /// Returns `self` / `rhs`, rounded tie to even.
    fn rounding_div(&self, rhs: &Self) -> Self;

    /// Returns `self` / `2ⁿ`, rounded tie to even.
    fn rounding_div_pow2(&self, n: u32) -> Self;

    /// Returns `self` % 2ⁿ, i.e. the n left-most bits of self.
    fn rem_pow2(&self, n: u32) -> Self;

    /// Returns `(self << shift, self >> (Self::BITS - shift))`
    fn widening_shl(&self, shift: u32) -> (Self, Self) {
        debug_assert!(shift < Self::BITS);
        match shift {
            0 => (*self, Self::ZERO),
            _ => (*self << shift, *self >> (Self::BITS - shift)),
        }
    }

    /// Returns `((self << shift) | carry, self >> (Self::BITS - shift))`
    fn carrying_shl(&self, shift: u32, carry: &Self) -> (Self, Self) {
        debug_assert!(shift < Self::BITS);
        debug_assert!(carry < &(Self::ONE << shift));
        ((*self << shift) | *carry, *self >> (Self::BITS - shift))
    }

    /// Returns `(self >> shift, self << (Self::BITS - shift))`
    fn widening_shr(&self, shift: u32) -> (Self, Self) {
        debug_assert!(shift < Self::BITS);
        match shift {
            0 => (*self, Self::ZERO),
            _ => (*self >> shift, *self << (Self::BITS - shift)),
        }
    }

    /// Returns `(carry | (self >> shift), self << (Self::BITS - shift))`
    fn carrying_shr(&self, shift: u32, carry: &Self) -> (Self, Self) {
        debug_assert!(shift < Self::BITS);
        debug_assert!((*carry << shift) == Self::ZERO);
        (*carry | (*self >> shift), *self << (Self::BITS - shift))
    }
}

/// Multiply x by 10 and add decimal digit (inplace).
pub(crate) fn imul10_add(x: &mut U256, d: u8) {
    debug_assert!(
        *x <= U256::new(
            0x19999999999999999999999999999999_u128,
            0x99999999999999999999999999999999_u128
        )
    );
    debug_assert!(d < 10);
    let ll = u128_lo(x.lo.0);
    let lh = u128_hi(x.lo.0);
    let hl = u128_lo(x.hi.0);
    let hh = u128_hi(x.hi.0);
    let mut t = ll * 10 + d as u128;
    let mut lo = u128_lo(t);
    t = lh * 10 + u128_hi(t);
    lo += t << 64;
    t = hl * 10 + u128_hi(t);
    let mut hi = u128_lo(t);
    t = hh * 10 + u128_hi(t);
    hi += t << 64;
    *x = U256::new(hi, lo);
}

/// Returns x / 10ⁿ, rounded tie to even.
#[allow(clippy::integer_division)]
pub(crate) fn rounding_div_pow10(x: &U512, n: u32) -> U512 {
    const CHUNK_SIZE: u32 = 38;
    const CHUNK_BASE: u128 = 10_u128.pow(CHUNK_SIZE);
    debug_assert_ne!(n, 0);
    let mut q = *x;
    let mut r = 0_u128;
    if n <= CHUNK_SIZE {
        let d = 10_u128.pow(n);
        (q, r) = q.div_rem(d);
        let tie = d >> 1;
        if r > tie || (r == tie && q.is_odd()) {
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
        if r > tie || (r == tie && (q.is_odd() || !all_chunks_zero)) {
            q.incr();
        }
    }
    q
}
