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
    HI_EXP_MASK, U256,
};

// Cut-off for small values
// 4.34011792384841269241918479487309437935796941860057715139559227691773315e-36
const SMALL_CUT_OFF: f256 = f256 {
    bits: U256::new(
        0x3ff89713765fce269de05bbe5d2df6f0,
        0xb6f406126cab80a1f5eca809c5595b15,
    ),
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
        // If |self| is very small, atan self = self.
        if abs_bits_self <= SMALL_CUT_OFF.bits {
            return *self;
        }
        // Now we have ε < |self| < 1
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
    fn calc_small_cutoff() {
        let mut lf = f256::from(1e-36_f64);
        let mut uf = f256::from(1e-35_f64);
        assert_eq!(lf, lf.asin());
        assert_ne!(uf, uf.asin());
        let mut f = (lf + uf) / f256::TWO;
        while lf < f && f < uf {
            if f == f.asin() {
                lf = f;
            } else {
                uf = f;
            }
            f = (lf + uf) / f256::TWO;
            if f == uf {
                f = lf;
            }
        }
        // println!("\n{lf:?}\n{:?}", lf.asin());
        // println!("\n{f:?}\n{:?}", f.asin());
        // println!("\n{uf:?}\n{:?}", uf.asin());
        // println!("\n// {f:e}");
        // println!("const SMALL_CUT_OFF: f256 = f256 {{");
        // println!("    bits: U256::new(");
        // println!(
        //     "        0x{:032x},\n        0x{:032x},\n    ),\n}};",
        //     f.bits.hi.0, f.bits.lo.0
        // );

        assert_eq!(f, f.asin());
        assert_eq!(f, SMALL_CUT_OFF);
        let g = f + f.ulp();
        assert_ne!(g, g.asin());
    }

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
