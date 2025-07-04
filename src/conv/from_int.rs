// ---------------------------------------------------------------------------
// Copyright:   (c) 2022 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use crate::{
    f256, BigUInt, EXP_BIAS, EXP_BITS, FRACTION_BITS, HI_FRACTION_BITS,
    SIGNIFICAND_BITS, U256,
};

impl f256 {
    /// Construct a finite `f256` from a signed 64-bit integer.
    #[must_use]
    #[inline]
    #[allow(clippy::cast_possible_wrap)]
    pub(crate) const fn from_i64(i: i64) -> Self {
        if i == 0 {
            return Self::ZERO;
        }
        // First cast to i128, because i64::MIN.abs() causes overflow.
        let j = (i as i128).unsigned_abs();
        let msb = 127 - j.leading_zeros();
        Self::new(
            i.is_negative() as u32,
            msb as i32,
            U256::new(j << (HI_FRACTION_BITS - msb), 0),
        )
    }

    /// Construct a finite `f256` from a signed 128-bit integer.
    #[allow(clippy::cast_sign_loss)]
    #[must_use]
    #[inline]
    #[allow(clippy::cast_possible_wrap)]
    pub(crate) const fn from_i128(i: i128) -> Self {
        if i == 0 {
            return Self::ZERO;
        }
        let j = match i.checked_abs() {
            Some(k) => k as u128,
            None => i as u128,
        };
        let msb = 127 - j.leading_zeros();
        Self::new(
            i.is_negative() as u32,
            msb as i32,
            U256::new(0, j).shift_left(FRACTION_BITS - msb),
        )
    }

    /// Construct a finite `f256` from an unsigned 64-bit integer.
    #[must_use]
    #[inline]
    #[allow(clippy::cast_possible_wrap)]
    pub(crate) const fn from_u64(i: u64) -> Self {
        if i == 0 {
            return Self::ZERO;
        }
        let msb = 127 - (i as u128).leading_zeros();
        Self::new(
            0_u32,
            msb as i32,
            U256::new((i as u128) << (HI_FRACTION_BITS - msb), 0),
        )
    }

    /// Construct a finite `f256` from an unsigned 128-bit integer.
    #[must_use]
    #[inline]
    #[allow(clippy::cast_possible_wrap)]
    pub(crate) const fn from_u128(i: u128) -> Self {
        if i == 0 {
            return Self::ZERO;
        }
        let msb = 127 - i.leading_zeros();
        Self::new(
            0_u32,
            msb as i32,
            U256::new(0, i).shift_left(FRACTION_BITS - msb),
        )
    }

    /// Construct a finite `f256` from an unsigned 256-bit integer.
    #[must_use]
    #[inline]
    #[allow(clippy::cast_possible_wrap)]
    pub(crate) fn from_u256(i: &U256) -> Self {
        debug_assert!(i.hi.0.leading_zeros() >= EXP_BITS);
        if i.is_zero() {
            return Self::ZERO;
        }
        let msb = i.msb();
        Self::new(0_u32, msb as i32, i.shift_left(FRACTION_BITS - msb))
    }
}

macro_rules! impl_from_signed_int {
    () => {
        impl_from_signed_int!(i8, i16, i32, i64);
    };
    ($($t:ty),*) => {
        $(
        impl From<$t> for f256 {
            #[allow(trivial_numeric_casts)]
            fn from(i: $t) -> Self {
                Self::from_i64(i as i64)
            }
        }
        )*
    }
}

impl_from_signed_int!();

impl From<i128> for f256 {
    fn from(i: i128) -> Self {
        Self::from_i128(i)
    }
}

#[cfg(test)]
mod from_signed_int_tests {
    use super::*;

    #[allow(clippy::cast_sign_loss)]
    fn check_from_signed_int<T>(numbers: &[T])
    where
        T: Into<i128> + Copy,
        f256: From<T>,
    {
        for n in numbers {
            let f = f256::from(*n);
            let i = (*n).into();
            let j = match i.checked_abs() {
                Some(k) => k as u128,
                None => i as u128,
            };
            assert_eq!(f.is_sign_negative(), i.is_negative());
            let (s, t, c) = f.decode();
            assert_eq!(c.hi.0, 0);
            assert_eq!(c.lo.0, j >> t as usize);
        }
    }

    #[test]
    fn test_from_i8() {
        let numbers: [i8; 7] = [-128, -38, -1, 0, 1, 28, 127];
        check_from_signed_int::<i8>(&numbers);
    }

    #[test]
    fn test_from_i16() {
        let numbers: [i16; 5] = [i16::MIN, -1, 0, 28200, i16::MAX];
        check_from_signed_int::<i16>(&numbers);
    }

    #[test]
    fn test_from_i32() {
        let numbers: [i32; 5] = [i32::MIN, -1, 0, 2000000, i32::MAX];
        check_from_signed_int::<i32>(&numbers);
    }

    #[test]
    fn test_from_i64() {
        let numbers: [i64; 5] = [i64::MIN, -1, 0, 2128255, i64::MAX];
        check_from_signed_int::<i64>(&numbers);
    }

    #[test]
    fn test_from_i128() {
        let numbers: [i128; 5] =
            [i128::MIN, -1, 0, 5_i128.pow(28), i128::MAX];
        check_from_signed_int::<i128>(&numbers);
    }
}

macro_rules! impl_from_unsigned_int {
    () => {
        impl_from_unsigned_int!(u8, u16, u32, u64);
    };
    ($($t:ty),*) => {
        $(
        impl From<$t> for f256 {
            #[allow(trivial_numeric_casts)]
            fn from(i: $t) -> Self {
                Self::from_u64(i as u64)
            }
        }
        )*
    }
}

impl_from_unsigned_int!();

impl From<u128> for f256 {
    fn from(i: u128) -> Self {
        Self::from_u128(i)
    }
}

#[cfg(test)]
mod from_unsigned_int_tests {
    use super::*;

    #[allow(clippy::cast_sign_loss)]
    fn check_from_unsigned_int<T>(numbers: &[T])
    where
        T: Into<u128> + Copy,
        f256: From<T>,
    {
        for n in numbers {
            let f = f256::from(*n);
            let i = (*n).into();
            assert!(f.is_sign_positive());
            let (s, t, c) = f.decode();
            assert_eq!(c.hi.0, 0);
            assert_eq!(c.lo.0, i >> t as usize);
        }
    }

    #[test]
    fn test_from_u8() {
        let numbers: [u8; 4] = [0, 1, 98, u8::MAX];
        check_from_unsigned_int::<u8>(&numbers);
    }

    #[test]
    fn test_from_u16() {
        let numbers: [u16; 3] = [0, 28200, u16::MAX];
        check_from_unsigned_int::<u16>(&numbers);
    }

    #[test]
    fn test_from_u32() {
        let numbers: [u32; 3] = [0, 2000000, u32::MAX];
        check_from_unsigned_int::<u32>(&numbers);
    }

    #[test]
    fn test_from_u64() {
        let numbers: [u64; 3] = [0, 2128255, u64::MAX];
        check_from_unsigned_int::<u64>(&numbers);
    }

    #[test]
    fn test_from_u128() {
        let numbers: [u128; 3] = [0, 7_u128.pow(27), u128::MAX];
        check_from_unsigned_int::<u128>(&numbers);
    }
}
