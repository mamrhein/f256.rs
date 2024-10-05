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
    consts::FRAC_PI_2,
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

const FP509_FRAC_PI_2: FP509 = FP509::new(
    0x3243f6a8885a308d313198a2e0370734,
    0x4a4093822299f31d0082efa98ec4e6c8,
    0x9452821e638d01377be5466cf34e90c6,
    0xcc0ac29b7c97c50dd3f84d5b5b547091,
);
const FP509_NEG_FRAC_PI_2: FP509 = FP509::new(
    0xcdbc095777a5cf72cece675d1fc8f8cb,
    0xb5bf6c7ddd660ce2ff7d1056713b1937,
    0x6bad7de19c72fec8841ab9930cb16f39,
    0x33f53d6483683af22c07b2a4a4ab8f6f,
);
const FP509_FRAC_PI_4: FP509 = FP509::new(
    0x1921fb54442d18469898cc51701b839a,
    0x252049c1114cf98e804177d4c7627364,
    0x4a29410f31c6809bbdf2a33679a74863,
    0x6605614dbe4be286e9fc26adadaa3849,
);
const FP509_NEG_FRAC_PI_4: FP509 = FP509::new(
    0xe6de04abbbd2e7b9676733ae8fe47c65,
    0xdadfb63eeeb306717fbe882b389d8c9b,
    0xb5d6bef0ce397f64420d5cc98658b79c,
    0x99fa9eb241b41d791603d9525255c7b7,
);

/// Computes the arctangent of a number (in radians).
fn atan(x: &BigFloat) -> FP509 {
    let x_abs = x.abs();
    let sign = (x.signum < 0) as usize;
    if x_abs < BigFloat::ONE {
        approx_atan(&FP509::from(x))
    } else if x_abs > BigFloat::ONE {
        // atan(±x) = ±½π - atan(1/x) for |x| > 1
        let xr = x.recip();
        let mut atan = [FP509_FRAC_PI_2, FP509_NEG_FRAC_PI_2][sign];
        atan -= &approx_atan(&FP509::from(&xr));
        atan
    } else {
        // atan(±1) = ±¼π
        [FP509_FRAC_PI_4, FP509_NEG_FRAC_PI_4][sign]
    }
}

impl f256 {
    /// Computes the arcsinus of a number (in radians).
    ///
    /// Return value is in radians in the range [-½π, ½π].
    pub fn asin(&self) -> Self {
        let abs_bits_self = abs_bits(self);
        // if self is NAN or |self| > 1, asin self is NAN
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
        Self::from(&atan(&x))
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
