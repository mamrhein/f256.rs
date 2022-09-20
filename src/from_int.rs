// ---------------------------------------------------------------------------
// Copyright:   (c) 2022 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use crate::{f256, Float256Repr};

macro_rules! impl_from_signed_int {
    () => {
        impl_from_signed_int!(i8, i16, i32, i64);
    };
    ($($t:ty),*) => {
        $(
        impl From<$t> for f256 {
            #[allow(trivial_numeric_casts)]
            fn from(i: $t) -> Self {
                Self{
                    repr: Float256Repr::from_i64(i as i64)
                }
            }
        }
        )*
    }
}

impl_from_signed_int!();

impl From<i128> for f256 {
    fn from(i: i128) -> Self {
        Self {
            repr: Float256Repr::from_i128(i),
        }
    }
}

// impl From<i128> for f256 {
//     fn from(i: i128) -> Self {
//         Self {
//             repr: Float256Repr::from_i128(i),
//         }
//     }
// }

// macro_rules! impl_from_uint {
//     () => {
//         impl_from_int!(u8, u16, u32, u64);
//     };
//     ($($t:ty),*) => {
//         $(
//         impl From<$t> for f256 {
//             #[inline]
//             fn from(i: $t) -> Self
//             {
//                 Decimal { coeff: i128::from(i), n_frac_digits: 0 }
//             }
//         }
//         )*
//     }
// }

#[cfg(test)]
mod from_signed_int_tests {
    use super::*;

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
            let (s, t, c) = f.repr.decode();
            assert_eq!(c.hi, 0);
            assert_eq!(c.lo, j >> t as usize);
        }
    }

    // #[test]
    // fn test_from_u8() {
    //     let numbers: [u8; 4] = [0, 1, 28, 255];
    //     check_from_int::<u8>(&numbers);
    // }

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
        let numbers: [i128; 5] = [i128::MIN, -1, 0, 5_i128.pow(28), i128::MAX];
        check_from_signed_int::<i128>(&numbers);
    }
}
