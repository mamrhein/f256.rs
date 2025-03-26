// ---------------------------------------------------------------------------
// Copyright:   (c) 2024 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use alloc::{vec, vec::Vec};
use core::{
    fmt,
    ops::{
        Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, Div, Mul,
        MulAssign, Rem, Shl, ShlAssign, Shr, ShrAssign, Sub, SubAssign,
    },
};

use super::{BigUInt, DivRem, HiLo};

#[inline(always)]
pub(crate) const fn u128_hi(u: u128) -> u128 {
    u >> 64
}

#[inline(always)]
pub(crate) const fn u128_lo(u: u128) -> u128 {
    u & 0xffffffffffffffff
}

#[derive(Clone, Copy, Default, Eq, Ord, PartialOrd, PartialEq)]
pub(crate) struct U128(pub(crate) u128);

impl U128 {
    /// Returns a new instance of Self.
    #[inline(always)]
    pub(crate) const fn new(i: u128) -> Self {
        Self(i)
    }
}

impl HiLo for U128 {
    type T = u64;
    // Number of u128 chunks in Self
    const N_CHUNKS: usize = 1;

    #[inline(always)]
    fn from_hi_lo(hi: Self::T, lo: Self::T) -> Self {
        Self(u128::from(hi) << 64 | u128::from(lo))
    }

    #[inline(always)]
    fn hi_t(&self) -> Self::T {
        u128_hi(self.0) as u64
    }

    #[inline(always)]
    fn lo_t(&self) -> Self::T {
        u128_lo(self.0) as u64
    }

    #[inline(always)]
    fn hi(&self) -> Self {
        Self(u128_hi(self.0))
    }

    #[inline(always)]
    fn lo(&self) -> Self {
        Self(u128_lo(self.0))
    }

    fn as_vec_u128(&self) -> Vec<u128> {
        vec![self.0]
    }
}

impl BigUInt for U128 {
    const ZERO: Self = Self(0_u128);
    const ONE: Self = Self(1_u128);
    const TWO: Self = Self(2_u128);
    const MAX: Self = Self(u128::MAX);
    const TIE: Self = Self(1_u128 << 127);

    #[inline(always)]
    fn is_even(&self) -> bool {
        (self.0 & 1) == 0
    }

    #[inline(always)]
    fn is_odd(&self) -> bool {
        (self.0 & 1) == 1
    }

    #[inline(always)]
    fn leading_zeros(&self) -> u32 {
        self.0.leading_zeros()
    }

    #[inline(always)]
    fn trailing_zeros(&self) -> u32 {
        self.0.trailing_zeros()
    }

    /// Add 1 to `self` inplace, wrapping around at Self::MAX.
    #[inline(always)]
    fn incr(&mut self) {
        self.0 = self.0.wrapping_add(1);
    }

    /// Subtract 1 from `self` inplace, wrapping around at Self::ZERO.
    #[inline(always)]
    fn decr(&mut self) {
        self.0 = self.0.wrapping_sub(1);
    }

    #[inline(always)]
    fn overflowing_add(&self, rhs: &Self) -> (Self, bool) {
        let (t, ovfl) = self.0.overflowing_add(rhs.0);
        (Self(t), ovfl)
    }

    #[inline(always)]
    fn wrapping_add(&self, rhs: &Self) -> Self {
        Self(self.0.wrapping_add(rhs.0))
    }

    #[inline(always)]
    fn overflowing_sub(&self, rhs: &Self) -> (Self, bool) {
        let (t, ovfl) = self.0.overflowing_sub(rhs.0);
        (Self(t), ovfl)
    }

    #[inline(always)]
    fn wrapping_sub(&self, rhs: &Self) -> Self {
        Self(self.0.wrapping_sub(rhs.0))
    }

    #[inline(always)]
    fn overflowing_mul(&self, rhs: &Self) -> (Self, bool) {
        let (t, ovfl) = self.0.overflowing_mul(rhs.0);
        (Self(t), ovfl)
    }

    // TODO: reimplement when [feature(bigint_helper_methods)] got stable.
    fn widening_mul(&self, rhs: &Self) -> (Self, Self) {
        let xh = u128_hi(self.0);
        let xl = u128_lo(self.0);
        let yh = u128_hi(rhs.0);
        let yl = u128_lo(rhs.0);
        let mut rl = xl * yl;
        let t1 = xl * yh;
        let t2 = xh * yl;
        let mut rh = xh * yh;
        let (t1, mut carry) = t1.overflowing_add(t2);
        rh += ((carry as u128) << 64) + u128_hi(t1);
        (rl, carry) = rl.overflowing_add(u128_lo(t1) << 64);
        rh += carry as u128;
        (Self(rl), Self(rh))
    }

    fn rounding_div(&self, rhs: &Self) -> Self {
        let (mut quot, rem) = self.div_rem(rhs);
        let tie = rhs.0 >> 1;
        if rem.0 > tie || (rem.0 == tie && (quot.0 & 1) == 1) {
            quot.incr();
        }
        quot
    }

    fn rounding_div_pow2(&self, n: u32) -> Self {
        const TIE: u128 = 1_u128 << 127;
        let (mut quot, rem) = self.widening_shr(n);
        if rem.0 > TIE || (rem.0 == TIE && (quot.0 & 1) == 1) {
            quot.incr();
        }
        quot
    }

    fn rem_pow2(&self, n: u32) -> Self {
        Self((self.0 << n) >> n)
    }
}

macro_rules! wrap_bin_op {
    (impl $imp:ident, $method:ident, $wrapper:ident) => {
        impl $imp for $wrapper {
            type Output = Self;
            #[inline(always)]
            fn $method(self, rhs: Self) -> Self::Output {
                Self(self.0.$method(rhs.0))
            }
        }
        impl $imp for &$wrapper {
            type Output = $wrapper;
            #[inline(always)]
            fn $method(self, rhs: Self) -> Self::Output {
                $wrapper(self.0.$method(rhs.0))
            }
        }
    };
    (impl $imp:ident, $method:ident, $wrapper:ident, $rhs:ident) => {
        impl $imp<$rhs> for $wrapper {
            type Output = Self;
            #[inline(always)]
            fn $method(self, rhs: $rhs) -> Self::Output {
                Self(self.0.$method(rhs))
            }
        }
        impl $imp<$rhs> for &$wrapper {
            type Output = $wrapper;
            #[inline(always)]
            fn $method(self, rhs: $rhs) -> Self::Output {
                $wrapper(self.0.$method(rhs))
            }
        }
    };
}

wrap_bin_op!(impl Add, add, U128);
wrap_bin_op!(impl Add, add, U128, u128);
wrap_bin_op!(impl BitAnd, bitand, U128);
wrap_bin_op!(impl BitOr, bitor, U128);
wrap_bin_op!(impl Div, div, U128);
wrap_bin_op!(impl Mul, mul, U128);
wrap_bin_op!(impl Shl, shl, U128, u32);
wrap_bin_op!(impl Shr, shr, U128, u32);
wrap_bin_op!(impl Sub, sub, U128);
wrap_bin_op!(impl Rem, rem, U128);

macro_rules! wrap_op_assign {
    (impl $imp:ident, $method:ident, $wrapper:ident) => {
        impl<'a> $imp<&'a Self> for $wrapper {
            #[inline(always)]
            fn $method(&mut self, rhs: &Self) {
                self.0.$method(rhs.0);
            }
        }
    };
    (impl $imp:ident, $method:ident, $wrapper:ident, ref $rhs:ident) => {
        impl<'a> $imp<&'a $rhs> for $wrapper {
            #[inline(always)]
            #[allow(trivial_numeric_casts)]
            fn $method(&mut self, rhs: &'a $rhs) {
                self.0.$method(*rhs as u128);
            }
        }
    };
    (impl $imp:ident, $method:ident, $wrapper:ident, $rhs:ident) => {
        impl<'a> $imp<$rhs> for $wrapper {
            #[inline(always)]
            fn $method(&mut self, rhs: $rhs) {
                self.0.$method(rhs);
            }
        }
    };
}

wrap_op_assign!(impl AddAssign, add_assign, U128);
wrap_op_assign!(impl AddAssign, add_assign, U128, ref u128);
wrap_op_assign!(impl AddAssign, add_assign, U128, ref u64);
wrap_op_assign!(impl BitAndAssign, bitand_assign, U128);
wrap_op_assign!(impl BitOrAssign, bitor_assign, U128);
wrap_op_assign!(impl MulAssign, mul_assign, U128);
wrap_op_assign!(impl ShlAssign, shl_assign, U128, u32);
wrap_op_assign!(impl ShrAssign, shr_assign, U128, u32);
wrap_op_assign!(impl SubAssign, sub_assign, U128);

impl BitOrAssign<bool> for U128 {
    #[inline(always)]
    fn bitor_assign(&mut self, rhs: bool) {
        self.0 |= rhs as u128;
    }
}

impl Add<u64> for U128 {
    type Output = Self;

    #[inline(always)]
    fn add(self, rhs: u64) -> Self::Output {
        Self::new(self.0 + rhs as u128)
    }
}

impl<'a> Add<&'a u64> for U128 {
    type Output = Self;

    #[inline(always)]
    fn add(self, rhs: &'a u64) -> Self::Output {
        Self::new(self.0 + *rhs as u128)
    }
}

impl Sub<u64> for U128 {
    type Output = Self;

    #[inline(always)]
    fn sub(self, rhs: u64) -> Self::Output {
        Self::new(self.0 - rhs as u128)
    }
}

impl DivRem<u128> for U128 {
    type Output = (Self, u128);

    #[inline(always)]
    #[allow(clippy::integer_division)]
    fn div_rem(self, rhs: u128) -> Self::Output {
        (U128::new(self.0 / rhs), self.0 % rhs)
    }
}

impl<'a> DivRem<&'a u128> for &'a U128 {
    type Output = (U128, u128);

    #[inline(always)]
    #[allow(clippy::integer_division)]
    fn div_rem(self, rhs: &'a u128) -> Self::Output {
        (U128::new(self.0 / *rhs), self.0 % *rhs)
    }
}

impl DivRem for U128 {
    type Output = (Self, Self);

    #[inline(always)]
    #[allow(clippy::integer_division)]
    fn div_rem(self, rhs: Self) -> Self::Output {
        (
            U128::new(self.0 / u128::from(rhs)),
            U128::new(self.0 % u128::from(rhs)),
        )
    }
}

impl<'a> DivRem<&'a U128> for &'a U128 {
    type Output = (U128, U128);

    #[inline(always)]
    #[allow(clippy::integer_division)]
    fn div_rem(self, rhs: &'a U128) -> Self::Output {
        (
            U128::new(self.0 / u128::from(rhs)),
            U128::new(self.0 % u128::from(rhs)),
        )
    }
}

impl From<bool> for U128 {
    #[inline(always)]
    fn from(value: bool) -> Self {
        Self::new(value as u128)
    }
}

impl From<u128> for U128 {
    #[inline(always)]
    fn from(value: u128) -> Self {
        Self::new(value)
    }
}

impl<'a> From<&'a u128> for U128 {
    #[inline(always)]
    fn from(value: &'a u128) -> Self {
        Self::new(*value)
    }
}

impl From<U128> for u128 {
    #[inline(always)]
    fn from(value: U128) -> Self {
        value.0
    }
}

impl<'a> From<&'a U128> for u128 {
    #[inline(always)]
    fn from(value: &'a U128) -> Self {
        value.0
    }
}

impl<'a> From<&'a [u128]> for U128 {
    #[inline(always)]
    fn from(value: &'a [u128]) -> Self {
        match value.len() {
            1 => Self::new(value[0]),
            0 => Self::ZERO,
            _ => panic!("Can't create a U128 from more than 1 u128!"),
        }
    }
}

impl<'a> From<&'a Vec<u128>> for U128 {
    #[inline(always)]
    fn from(value: &'a Vec<u128>) -> Self {
        debug_assert!(value.len() == 1);
        Self::new(value[0])
    }
}

impl fmt::Debug for U128 {
    #[inline(always)]
    fn fmt(&self, form: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(form, "0x{:032x}", self.0)
    }
}

impl fmt::Display for U128 {
    #[inline(always)]
    fn fmt(&self, form: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(form)
    }
}

#[cfg(test)]
mod convert_tests {
    use super::*;

    #[test]
    fn test_from_to_u128() {
        let x = u128::MAX;
        let y = U128::from(&x);
        assert_eq!(y, U128::MAX);
        let z = u128::from(&y);
        assert_eq!(x, z);
    }

    #[test]
    fn test_from_to_vec() {
        let x = [u128::MAX];
        let y = U128::from(&x[..]);
        assert_eq!(y, U128::MAX);
        let z = y.as_vec_u128();
        assert_eq!(x, z[..]);
    }

    #[test]
    fn test_from_bool() {
        assert_eq!(U128::from(false), U128::ZERO);
        assert_eq!(U128::from(true), U128::ONE);
    }
}

#[cfg(test)]
mod divrem_tests {
    use super::*;

    #[test]
    fn test_divrem_self() {
        let x = U128::MAX;
        let y = U128::new(7_u128);
        let (q, r) = x.div_rem(y);
        assert_eq!(q, U128::new(x.0 / y.0));
        assert_eq!(r, U128::new(x.0 % y.0));
    }

    #[test]
    fn test_divrem_u128() {
        let x = U128::MAX;
        let y = 37_u128;
        let (q, r) = x.div_rem(y);
        assert_eq!(q, U128::new(x.0 / y));
        assert_eq!(r, x.0 % y);
    }
}

#[cfg(test)]
mod u128_widening_mul_tests {
    use super::*;

    #[test]
    fn test_max() {
        let x = U128::MAX;
        let z = x.widening_mul(&x);
        assert_eq!(z, (U128::ONE, U128(u128::MAX - 1)));
    }
}
