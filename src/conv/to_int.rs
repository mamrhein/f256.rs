// ---------------------------------------------------------------------------
// Copyright:   (c) 2025 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use crate::big_uint::{BigUInt, HiLo};
use crate::{f256, split_f256_enc};

impl TryFrom<&f256> for i32 {
    type Error = ();

    #[allow(clippy::cast_possible_wrap)]
    #[allow(clippy::cast_possible_truncation)]
    fn try_from(value: &f256) -> Result<Self, Self::Error> {
        let (sign, exp, signif) = split_f256_enc(value);
        let ntz = signif.trailing_zeros();
        match exp + ntz as Self {
            n @ 0..=30 => {
                let t = (signif >> exp.unsigned_abs()).lo_t().0 as Self;
                Ok([t, -t][sign as usize])
            }
            31 => [Err(()), Ok(Self::MIN)][sign as usize],
            256 => Ok(0),
            _ => Err(()),
        }
    }
}

#[cfg(test)]
mod to_i32_tests {
    use super::*;
    use crate::ONE_HALF;
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
    }

    #[test]
    fn test_err() {
        assert!(<&f256 as TryInto<i32>>::try_into(&f256::NAN).is_err());
        assert!(
            <&f256 as TryInto<i32>>::try_into(&f256::NEG_INFINITY).is_err()
        );
        assert!(<&f256 as TryInto<i32>>::try_into(&f256::INFINITY).is_err());
        assert!(<&f256 as TryInto<i32>>::try_into(&ONE_HALF).is_err());
        assert!(<&f256 as TryInto<i32>>::try_into(&f256::from(1234567.89))
            .is_err());
        assert!(<&f256 as TryInto<i32>>::try_into(&f256::from(
            i32::MAX as i64 + 1
        ))
        .is_err());
    }
}
