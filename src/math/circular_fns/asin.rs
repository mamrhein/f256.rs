// ---------------------------------------------------------------------------
// Copyright:   (c) 2024 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use crate::{
    abs_bits,
    consts::{FRAC_PI_2, FRAC_PI_4},
    f256,
    math::{
        big_float::BigFloat, circular_fns::approx_atan::approx_atan,
        fp509::FP509,
    },
    HI_EXP_MASK,
};

/// Computes the arctangent of a number (in radians).
fn atan(x: &BigFloat) -> f256 {
    let x_abs = x.abs();
    let sign = (x.signum < 0) as usize;
    if x_abs < BigFloat::ONE {
        f256::from(&approx_atan(&FP509::from(x)))
    } else if x_abs > BigFloat::ONE {
        // atan(±x) = ±½π - atan(1/x) for |x| > 1
        let xr = x.recip();
        let atan = [BigFloat::FRAC_PI_2, -BigFloat::FRAC_PI_2][sign]
            - &BigFloat::from(&approx_atan(&FP509::from(&xr)));
        f256::from(&atan)
    } else {
        // atan(±1) = ±¼π
        [FRAC_PI_4, -FRAC_PI_4][sign]
    }
}

impl f256 {
    /// Computes the arcsinus of a number (in radians).
    ///
    /// Return value is in radians in the range [-½π, ½π].
    pub fn asin(&self) -> Self {
        let abs_bits_self = abs_bits(self);
        // If self is NAN, asin self is NAN.
        if (abs_bits_self.hi.0 | (abs_bits_self.lo.0 != 0) as u128)
            > HI_EXP_MASK
        {
            return f256::NAN;
        }
        // if |self| > 1, asin self is NAN
        if abs_bits_self > f256::ONE.bits {
            return f256::NAN;
        }
        // asin(±1) = ±½π
        if abs_bits_self == f256::ONE.bits {
            return [FRAC_PI_2, -FRAC_PI_2][self.sign() as usize];
        }
        // Now we have |self| < 1
        // asin(x) = atan(x/√(1-x²))
        let mut x = BigFloat::from(self);
        x.idiv(&(BigFloat::ONE - &x.square()).sqrt());
        atan(&x)
    }
}

#[cfg(test)]
mod asin_tests {
    use super::*;
    use crate::{
        consts::{FRAC_PI_3, FRAC_PI_4, FRAC_PI_6, SQRT_2},
        ONE_HALF,
    };

    #[test]
    fn test_asin_inf() {
        assert!(f256::INFINITY.asin().is_nan());
        assert!(f256::NEG_INFINITY.asin().is_nan());
    }

    #[test]
    fn test_asin_gt_1() {
        assert!(f256::TWO.asin().is_nan());
        let x = f256::ONE + f256::EPSILON;
        assert!(x.asin().is_nan());
        let x = -x;
        assert!(x.asin().is_nan());
    }

    #[test]
    fn test_asin_zero() {
        assert_eq!(f256::ZERO.asin(), f256::ZERO);
        assert_eq!(f256::NEG_ZERO.asin(), f256::ZERO);
    }

    #[test]
    fn test_asin_one() {
        assert_eq!(f256::ONE.asin(), FRAC_PI_2);
        assert_eq!(f256::NEG_ONE.asin(), -FRAC_PI_2);
    }

    #[test]
    fn test_asin_one_half() {
        assert_eq!(ONE_HALF.asin(), FRAC_PI_6);
        assert_eq!((-ONE_HALF).asin(), -FRAC_PI_6);
    }

    #[test]
    fn test_asin_one_half_times_sqrt_2() {
        let x = ONE_HALF * SQRT_2;
        assert_eq!(x.asin(), FRAC_PI_4);
        assert_eq!((-x).asin(), -FRAC_PI_4);
    }

    #[test]
    fn test_asin_one_half_times_sqrt_3() {
        let x = ONE_HALF * f256::from(3).sqrt();
        assert_eq!(x.asin(), FRAC_PI_3);
        assert_eq!((-x).asin(), -FRAC_PI_3);
    }
}
