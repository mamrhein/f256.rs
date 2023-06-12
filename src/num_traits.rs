// ---------------------------------------------------------------------------
// Copyright:   (c) 2021 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use core::{ops::Add, str::FromStr};

use num_traits::{Num, One, Zero};

use crate::f256;

impl Zero for f256
where
    Self: Add<Output = Self>,
{
    #[inline(always)]
    fn zero() -> Self {
        Self::ZERO
    }

    #[inline(always)]
    fn is_zero(&self) -> bool {
        self.eq_zero()
    }
}

#[cfg(test)]
mod zero_tests {
    use super::*;

    #[test]
    fn test_zero() {
        assert!(f256::is_zero(&f256::zero()));
        assert!(f256::is_zero(&f256::from(0_u64)));
        assert!(!f256::is_zero(&f256::from(7)));
    }
}

impl One for f256 {
    /// Returns the multiplicative identity element of Self, Self::ONE.
    #[inline(always)]
    fn one() -> Self {
        Self::ONE
    }

    /// Returns true if self is equal to the multiplicative identity.
    #[inline(always)]
    fn is_one(&self) -> bool {
        *self == Self::ONE
    }
}

#[cfg(test)]
mod one_tests {
    use super::*;

    #[test]
    fn test_one() {
        assert!(f256::is_one(&f256::one()));
        assert!(f256::is_one(&f256::from(1)));
        assert!(!f256::is_one(&f256::from(3)));
    }
}

impl Num for f256 {
    type FromStrRadixErr = <Self as FromStr>::Err;

    fn from_str_radix(
        str: &str,
        radix: u32,
    ) -> Result<Self, Self::FromStrRadixErr> {
        if radix != 10 {
            // The internals of ParseFloatError are not public.
            // The following hack is used to return ParseFloatError::Invalid.
            return Err(f64::from_str("_").unwrap_err());
        }
        Self::from_str(str)
    }
}

#[cfg(test)]
mod from_str_radix_tests {

    use super::*;

    #[test]
    fn test_from_str_radix() {
        let f = f256::from_str_radix("-17.5", 10).unwrap();
        assert_eq!(f.as_sign_exp_signif(), (1, -1, (0, 35)));
    }

    #[test]
    fn test_err_invalid_radix() {
        let res = f256::from_str_radix("5.4", 16);
        assert!(res.is_err());
    }
}
