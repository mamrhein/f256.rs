// ---------------------------------------------------------------------------
// Copyright:   (c) 2025 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use crate::big_uint::{BigUInt, HiLo};
use crate::{
    abs_bits, exp, f256, signif, split_f256_enc, EMAX, FRACTION_BITS,
    HI_FRACTION_BIAS, HI_FRACTION_BITS, HI_FRACTION_MASK, SIGNIFICAND_BITS,
};
use core::fmt::{Display, Formatter};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum IntoIntError {
    NotInteger,
    OutOfRange,
}

impl Display for IntoIntError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NotInteger => "Given value is not an integer.".fmt(f),
            Self::OutOfRange => "Given value is out of range.".fmt(f),
        }
    }
}

#[allow(clippy::cast_possible_wrap)]
#[allow(clippy::cast_possible_truncation)]
fn try_into_int(value: &f256) -> Result<(u32, u128), IntoIntError> {
    let abs_bits = abs_bits(value);
    if abs_bits.is_zero() {
        return Ok((0_u32, 0_u128));
    }
    let exp = exp(&abs_bits);
    const EXP_LIM: i32 = SIGNIFICAND_BITS as i32;
    const SPECIAL_EXP: i32 = EMAX + 1;
    match exp {
        ..0 => Err(IntoIntError::NotInteger),
        0..EXP_LIM => {
            if exp
                < FRACTION_BITS.saturating_sub(abs_bits.trailing_zeros())
                    as i32
            {
                Err(IntoIntError::NotInteger)
            } else {
                let t = signif(&abs_bits) >> FRACTION_BITS - exp as u32;
                if t.hi.is_zero() {
                    Ok((value.sign(), t.lo.0))
                } else {
                    Err(IntoIntError::OutOfRange)
                }
            }
        }
        SPECIAL_EXP => Err(IntoIntError::NotInteger),
        _ => {
            // An int, but too large
            Err(IntoIntError::OutOfRange)
        }
    }
}

fn try_into_i128(value: &f256) -> Result<i128, IntoIntError> {
    let (sign, abs) = try_into_int(value)?;
    let res = if sign == 0 {
        i128::try_from(abs).map_err(|_| IntoIntError::OutOfRange)
    } else {
        if abs <= i128::MAX as u128 {
            Ok(-(abs as i128))
        } else if abs == i128::MAX as u128 + 1 {
            Ok(i128::MIN)
        } else {
            Err(IntoIntError::OutOfRange)
        }
    };
    res
}

macro_rules! impl_try_from_f256_for_signed_int {
    () => {
        impl_try_from_f256_for_signed_int!(i8, i16, i32, i64, i128, isize);
    };
    ($($t:ty),*) => {
        $(
        impl TryFrom<&f256> for $t {
            type Error = IntoIntError;

            fn try_from(value: &f256) -> Result<Self, Self::Error> {
                let t = try_into_i128(value)?;
                Self::try_from(t).map_err(|_| IntoIntError::OutOfRange)
            }
        }
        )*
    }
}

impl_try_from_f256_for_signed_int!();

fn try_into_u128(value: &f256) -> Result<u128, IntoIntError> {
    if value.is_sign_negative() && !value.eq_zero() {
        return Err(IntoIntError::OutOfRange);
    }
    let u = try_into_int(value)?;
    Ok(u.1)
}

macro_rules! impl_try_from_f256_for_unsigned_int {
    () => {
        impl_try_from_f256_for_unsigned_int!(u8, u16, u32, u64, u128, usize);
    };
    ($($t:ty),*) => {
        $(
        impl TryFrom<&f256> for $t {
            type Error = IntoIntError;

            fn try_from(value: &f256) -> Result<Self, Self::Error> {
                let t = try_into_u128(value)?;
                Self::try_from(t).map_err(|_| IntoIntError::OutOfRange)
            }
        }
        )*
    }
}

impl_try_from_f256_for_unsigned_int!();

#[cfg(test)]
mod to_i32_tests {
    use super::*;
    use crate::{HI_FRACTION_BIAS, HI_FRACTION_MASK, ONE_HALF};
    use core::ops::Neg;

    #[test]
    fn test_ok() {
        assert_eq!((&f256::ZERO).try_into(), Ok(0_i32));
        assert_eq!((&f256::TWO).try_into(), Ok(2_i32));
        assert_eq!((&f256::TEN.neg()).try_into(), Ok(-10_i32));
        let max = f256::ONE.mul_pow2(31) - f256::ONE;
        assert_eq!((&max).try_into(), Ok(i32::MAX));
        let mut min = f256::ONE.mul_pow2(31).neg();
        assert_eq!((&min).try_into(), Ok(i32::MIN));
        assert_eq!((&(min + f256::ONE)).try_into(), Ok(i32::MIN + 1));
        assert_eq!((&(min.div2())).try_into(), Ok(i32::MIN / 2));
        assert_eq!(
            (&(min.div2() + f256::ONE)).try_into(),
            Ok(i32::MIN / 2 + 1)
        );
        let f = f256::NEG_ZERO;
        assert_eq!(i32::try_from(&f).unwrap(), 0_i32);
    }

    #[test]
    fn test_non_integer() {
        assert_eq!(
            <&f256 as TryInto<i32>>::try_into(&f256::NAN),
            Err(IntoIntError::NotInteger)
        );
        assert_eq!(
            <&f256 as TryInto<i32>>::try_into(&f256::NEG_INFINITY),
            Err(IntoIntError::NotInteger)
        );
        assert_eq!(
            <&f256 as TryInto<i32>>::try_into(&f256::INFINITY),
            Err(IntoIntError::NotInteger)
        );
        assert_eq!(
            <&f256 as TryInto<i32>>::try_into(&ONE_HALF),
            Err(IntoIntError::NotInteger)
        );
        assert_eq!(
            <&f256 as TryInto<i32>>::try_into(&f256::from(1234567.89)),
            Err(IntoIntError::NotInteger)
        );
        let f = f256::from_sign_exp_signif(
            1,
            25 - FRACTION_BITS as i32,
            (HI_FRACTION_BIAS + HI_FRACTION_MASK, u128::MAX),
        );
        assert_eq!(i32::try_from(&f), Err(IntoIntError::NotInteger));
    }

    #[test]
    fn test_out_of_range() {
        assert_eq!(
            <&f256 as TryInto<i32>>::try_into(&f256::from(
                i32::MAX as i64 + 1
            )),
            Err(IntoIntError::OutOfRange)
        );
        let f = -f256::from(i32::MIN);
        assert_eq!(i32::try_from(&f), Err(IntoIntError::OutOfRange));
        let f = f256::from_sign_exp_signif(
            1,
            256,
            (HI_FRACTION_BIAS + HI_FRACTION_MASK, u128::MAX),
        );
        assert_eq!(i32::try_from(&f), Err(IntoIntError::OutOfRange));
    }
}

#[cfg(test)]
mod to_i128_tests {
    use super::*;

    #[test]
    fn test_ok() {
        let f = f256::from(i128::MIN);
        assert_eq!(i128::try_from(&f).unwrap(), i128::MIN);
        let f = f256::from(i128::MAX);
        assert_eq!(i128::try_from(&f).unwrap(), i128::MAX);
    }

    #[test]
    fn test_out_of_range() {
        let f = -f256::from(i128::MIN);
        assert_eq!(i128::try_from(&f), Err(IntoIntError::OutOfRange));
    }
}

#[cfg(test)]
mod to_u64_tests {
    use super::*;

    #[test]
    fn test_ok() {
        let f = f256::from(u64::MIN);
        assert_eq!(u64::try_from(&f).unwrap(), u64::MIN);
        let f = f256::from(u64::MAX);
        assert_eq!(u64::try_from(&f).unwrap(), u64::MAX);
        let f = f256::NEG_ZERO;
        assert_eq!(u64::try_from(&f).unwrap(), 0_u64);
    }

    #[test]
    fn test_out_of_range() {
        let f = f256::from(u64::MAX) + f256::ONE;
        assert_eq!(u64::try_from(&f), Err(IntoIntError::OutOfRange));
        let f = f256::NEG_ONE;
        assert_eq!(u64::try_from(&f), Err(IntoIntError::OutOfRange));
    }
}
