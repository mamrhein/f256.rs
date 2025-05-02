// ---------------------------------------------------------------------------
// Copyright:   (c) 2023 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use super::exp::approx_exp;
use super::log::approx_ln;
use super::Float512;
use crate::{abs_bits, f256, BinEncSpecial};

const ONE_THIRD: Float512 = Float512::new(
    1,
    -2,
    &[
        0x55555555555555555555555555555555_u128,
        0x55555555555555555555555555555555_u128,
        0x55555555555555555555555555555555_u128,
        0x55555555555555555555555555555556_u128,
    ],
);

impl f256 {
    /// Returns the cube root of `self`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use f256::f256;
    /// let f = f256::from(2062933417);
    /// assert_eq!(f.cbrt(), f256::from(1273));
    /// assert_eq!(f256::NEG_ONE.cbrt(), f256::NEG_ONE);
    /// assert_eq!(f256::NEG_ZERO.cbrt(), f256::NEG_ZERO);
    /// ```
    #[must_use]
    pub fn cbrt(self) -> Self {
        let abs_bits = abs_bits(&self);
        if abs_bits.is_special() || abs_bits == Self::ONE.bits {
            //`self` is either not a number, infinite or equal to zero or one.
            return self;
        }
        // `self` is finite and != 0
        let x = Float512::from(&self);
        // ³√|x| = eʷ with w = ⅓⋅logₑ |x|
        let mut r = approx_exp(&(ONE_THIRD * approx_ln(&x.abs())));
        r.copy_sign(&x);
        Self::from(&r)
    }
}

#[cfg(test)]
mod cbrt_tests {
    use super::*;
    use crate::big_uint::U256;
    use crate::fast_mul;
    use core::str::FromStr;

    #[test]
    fn test_special_values() {
        assert!(f256::NAN.cbrt().is_nan());
        for val in [
            f256::NEG_INFINITY,
            f256::NEG_ONE,
            f256::NEG_ZERO,
            f256::ZERO,
            f256::ONE,
            f256::INFINITY,
        ] {
            assert_eq!(val.cbrt(), val);
        }
    }

    #[test]
    fn test_exact_cubes() {
        let f = f256::from(343);
        assert_eq!(f.cbrt(), f256::from(7));
        let f = f256::from(27) / f256::from(8);
        assert_eq!(f.cbrt(), f256::ONE + f256::ONE.div2());
        let f = f256::from_str("84807037425.375").unwrap();
        assert_eq!(f.cbrt(), f256::from_str("4393.5").unwrap());
    }

    #[test]
    fn test_cbrt_max() {
        let f = f256::MAX;
        let r = f.cbrt();
        assert_eq!(f, r.powi(3));
    }

    #[test]
    fn test_cbrt_min() {
        let f = f256::MIN;
        let r = f.cbrt();
        assert_eq!(f, r.powi(3));
    }

    #[test]
    fn test_subnormal() {
        let f = f256 {
            bits: U256::new(
                161381583805889998189973969922,
                288413346707470246106660640932215474040,
            ),
        };
        assert!(f.is_subnormal());
        let r = f.cbrt();
        assert!(r.is_normal());
        assert_eq!(r.powi(3), f);
    }

    #[test]
    fn test_near_zero() {
        let f = f256::MIN_GT_ZERO;
        let r = f.cbrt();
        assert!(r.is_normal());
        let (a, b) = fast_mul(&r, &r);
        let g = a.mul_add(r, b * r);
        assert_eq!(g, f);
    }
}
