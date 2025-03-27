// ---------------------------------------------------------------------------
// Copyright:   (c) 2024 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use super::{approx_atan::approx_atan, Float256, FP492};
use crate::{
    abs_bits,
    consts::{FRAC_PI_2, PI},
    f256, HI_EXP_MASK, U256,
};

const FP492_FRAC_PI_2: FP492 = FP492::new(
    0x00001921fb54442d18469898cc51701b,
    0x839a252049c1114cf98e804177d4c762,
    0x73644a29410f31c6809bbdf2a33679a7,
    0x48636605614dbe4be286e9fc26adadaa,
);
const FP492_NEG_FRAC_PI_2: FP492 = FP492::new(
    0xffffe6de04abbbd2e7b9676733ae8fe4,
    0x7c65dadfb63eeeb306717fbe882b389d,
    0x8c9bb5d6bef0ce397f64420d5cc98658,
    0xb79c99fa9eb241b41d791603d9525256,
);
const FP492_FRAC_PI_4: FP492 = FP492::new(
    0x00000c90fdaa22168c234c4c6628b80d,
    0xc1cd129024e088a67cc74020bbea63b1,
    0x39b22514a08798e3404ddef9519b3cd3,
    0xa431b302b0a6df25f14374fe1356d6d5,
);
const FP492_NEG_FRAC_PI_4: FP492 = FP492::new(
    0xfffff36f0255dde973dcb3b399d747f2,
    0x3e32ed6fdb1f77598338bfdf44159c4e,
    0xc64ddaeb5f78671cbfb22106ae64c32c,
    0x5bce4cfd4f5920da0ebc8b01eca9292b,
);

/// Computes the arctangent of a number (in radians).
fn atan(x: &Float256) -> FP492 {
    let x_abs = x.abs();
    let sign = (x.signum() < 0) as usize;
    if x_abs < Float256::ONE {
        approx_atan(&FP492::from(x))
    } else if x_abs > Float256::ONE {
        // atan(±x) = ±½π - atan(1/x) for |x| > 1
        let xr = x.recip();
        &[FP492_FRAC_PI_2, FP492_NEG_FRAC_PI_2][sign]
            - &approx_atan(&FP492::from(&xr))
    } else {
        // atan(±1) = ±¼π
        [FP492_FRAC_PI_4, FP492_NEG_FRAC_PI_4][sign]
    }
}

// Cut-off for small values
// 4.34011792384841269241918479487309437935796941860057715139559227691773315e-36
const SMALL_CUT_OFF_ASIN: f256 = f256 {
    bits: U256::new(
        0x3ff89713765fce269de05bbe5d2df6f0,
        0xb6f406126cab80a1f5eca809c5595b15,
    ),
};
// 8.6096782745546197660360502459384599237219217291093734171742066429506995e-72
const SMALL_CUT_OFF_ACOS: f256 = f256 {
    bits: U256::new(
        0x3ff12e6c89452821e638d01377be5466,
        0xcf34e90c6cc0ac29b7c97c50dd3f84d5,
    ),
};

impl f256 {
    /// Computes the arcsinus of a number (in radians).
    ///
    /// Return value is in radians in the range [-½π, ½π], or NaN if the
    /// number is outside the range [-1, 1].
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
        // If |self| is very small, asin self = self.
        if abs_bits_self <= SMALL_CUT_OFF_ASIN.bits {
            return *self;
        }
        // Now we have ε < |self| < 1
        // asin(x) = atan(x/√(1-x²))
        let mut x = Float256::from(self);
        x /= &(Float256::ONE - x.square()).sqrt();
        Self::from(&atan(&x))
    }

    /// Computes the arccosinus of a number (in radians).
    ///
    /// Return value is in radians in the range [0, π], or NaN if the number
    /// is outside the range [-1, 1].
    pub fn acos(&self) -> Self {
        let abs_bits_self = abs_bits(self);
        // if self is NAN or |self| > 1, acos self is NAN
        if abs_bits_self > f256::ONE.bits {
            return f256::NAN;
        }
        // acos(1) = 0, acos(-1) = π
        if abs_bits_self == f256::ONE.bits {
            return [Self::ZERO, PI][self.sign() as usize];
        }
        // If |self| is very small, acos self = ½π.
        // if abs_bits_self <= SMALL_CUT_OFF_ACOS.bits {
        //     return FRAC_PI_2;
        // }
        // Now we have ε < |self| < 1
        // acos(x) = ½π - asin(x) = ½π - atan(x/√(1-x²))
        let mut x = Float256::from(self);
        x /= &(Float256::ONE - x.square()).sqrt();
        Self::from(&(&FP492_FRAC_PI_2 - &atan(&x)))
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
        // println!("const SMALL_CUT_OFF_ASIN: f256 = f256 {{");
        // println!("    bits: U256::new(");
        // println!(
        //     "        0x{:032x},\n        0x{:032x},\n    ),\n}};",
        //     f.bits.hi.0, f.bits.lo.0
        // );

        assert_eq!(f, f.asin());
        assert_eq!(f, SMALL_CUT_OFF_ASIN);
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

#[cfg(test)]
mod acos_tests {
    use super::*;
    use crate::{
        consts::{FRAC_PI_3, FRAC_PI_4, FRAC_PI_6, SQRT_2},
        ONE_HALF,
    };

    #[test]
    fn calc_small_cutoff() {
        let mut lf = f256::from(1e-73_f64);
        let mut uf = f256::from(1e-71_f64);
        assert_eq!(lf.acos(), FRAC_PI_2);
        assert_ne!(uf.acos(), FRAC_PI_2);
        let mut f = (lf + uf) / f256::TWO;
        while lf < f && f < uf {
            if f.acos() == FRAC_PI_2 {
                lf = f;
            } else {
                uf = f;
            }
            f = (lf + uf) / f256::TWO;
            if f == uf {
                f = lf;
            }
        }
        // println!("\n{lf:?}\n{:?}", lf.acos());
        // println!("\n{f:?}\n{:?}", f.acos());
        // println!("\n{uf:?}\n{:?}", uf.acos());
        // println!("\n// {f:e}");
        // println!("const SMALL_CUT_OFF_ACOS: f256 = f256 {{");
        // println!("    bits: U256::new(");
        // println!(
        //     "        0x{:032x},\n        0x{:032x},\n    ),\n}};",
        //     f.bits.hi.0, f.bits.lo.0
        // );

        assert_eq!(f.acos(), FRAC_PI_2);
        assert_eq!(f, SMALL_CUT_OFF_ACOS);
        let g = f + f.ulp();
        assert_ne!(g.acos(), FRAC_PI_2);
    }

    #[test]
    fn test_acos_inf() {
        assert!(f256::INFINITY.acos().is_nan());
        assert!(f256::NEG_INFINITY.acos().is_nan());
    }

    #[test]
    fn test_acos_gt_1() {
        assert!(f256::TWO.acos().is_nan());
        let x = f256::ONE + f256::EPSILON;
        assert!(x.acos().is_nan());
        let x = -x;
        assert!(x.acos().is_nan());
    }

    #[test]
    fn test_acos_zero() {
        assert_eq!(f256::ZERO.acos(), FRAC_PI_2);
        assert_eq!(f256::NEG_ZERO.acos(), FRAC_PI_2);
    }

    #[test]
    fn test_acos_one() {
        assert_eq!(f256::ONE.acos(), f256::ZERO);
        assert_eq!(f256::NEG_ONE.acos(), PI);
    }

    #[test]
    fn test_acos_one_half() {
        assert_eq!(ONE_HALF.acos(), FRAC_PI_3);
        assert_eq!((-ONE_HALF).acos(), FRAC_PI_3 + FRAC_PI_3);
    }

    #[test]
    fn test_acos_one_half_times_sqrt_2() {
        let x = ONE_HALF * SQRT_2;
        assert!(x.acos().almost_eq::<1>(&FRAC_PI_4));
        assert_eq!((-x).acos(), PI - FRAC_PI_4);
    }

    #[test]
    fn test_acos_one_half_times_sqrt_3() {
        let x = ONE_HALF * f256::from(3).sqrt();
        assert!(x.acos().almost_eq::<1>(&FRAC_PI_6));
        assert!((-x).acos().almost_eq::<1>(&(PI - FRAC_PI_6)));
    }
}
