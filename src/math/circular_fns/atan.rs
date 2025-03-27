// ---------------------------------------------------------------------------
// Copyright:   (c) 2024 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use core::cmp::max;

use super::{approx_atan::approx_atan, Float256, FP492};
use crate::{
    abs_bits, abs_bits_sticky,
    consts::{FRAC_PI_2, FRAC_PI_4, PI},
    f256, sign_bits_hi, BinEncAnySpecial, EXP_BIAS, HI_EXP_MASK,
    HI_FRACTION_BITS, SIGNIFICAND_BITS, U256,
};

// Cut-off for large values (2²³⁷)
const LARGE_CUT_OFF: U256 = U256::new(
    ((EXP_BIAS + SIGNIFICAND_BITS) as u128) << HI_FRACTION_BITS,
    0_u128,
);

// Cut-off for small values
// 3.44474912109377383968972695618616830461246359669623725678161320654548683e-36
const SMALL_CUT_OFF: f256 = f256 {
    bits: U256::new(
        0x3ff89250bfe1b082f4f9b8d4ce85ca7f,
        0xcf68624f8a91d242f267bb52b5b4432a,
    ),
};

impl f256 {
    /// Computes the arctangent of a number (in radians).
    ///
    /// Return value is in radians in the range [-½π, ½π].
    pub fn atan(&self) -> Self {
        let abs_bits_self = abs_bits(self);
        // If self is NAN, atan self is NAN.
        if (abs_bits_self.hi.0 | (abs_bits_self.lo.0 != 0) as u128)
            > HI_EXP_MASK
        {
            return f256::NAN;
        }
        // If |self| >= 2²³⁷, atan self = ±½π.
        if abs_bits_self.hi >= LARGE_CUT_OFF.hi {
            let mut res = FRAC_PI_2;
            res.bits.hi.0 ^= sign_bits_hi(self);
            return res;
        }
        // If |self| is very small, atan self = self.
        if abs_bits_self <= SMALL_CUT_OFF.bits {
            return *self;
        }
        // Now we have ε < |self| < 2²³⁷.
        if abs_bits_self < Self::ONE.bits {
            Self::from(&approx_atan(&FP492::from(self)))
        } else if abs_bits_self > Self::ONE.bits {
            // atan(±x) = ±½π - atan(1/x) for |x| > 1
            let xr = Float256::from(self).recip();
            let atan = [Float256::FRAC_PI_2, -Float256::FRAC_PI_2]
                [self.sign() as usize]
                - Float256::from(&approx_atan(&FP492::from(&xr)));
            Self::from(&atan)
        } else {
            // atan(±1) = ±¼π
            [FRAC_PI_4, -FRAC_PI_4][self.sign() as usize]
        }
    }

    /// Computes the four quadrant arctangent of `self` (`y`) and `other`
    /// (`x`) in radians.
    ///
    /// * `x = 0`, `y = 0`: `0`
    /// * `x >= 0`: `arctan(y/x)` -> `[-½π, ½π]`
    /// * `y >= 0`: `arctan(y/x) + π` -> `(½π, π]`
    /// * `y < 0`: `arctan(y/x) - π` -> `(-π, -½π)`
    pub fn atan2(&self, other: &Self) -> Self {
        let mut abs_bits_x = abs_bits(&other);
        let mut abs_bits_y = abs_bits(&self);
        // Check whether one or both operands are NaN, infinite or zero.
        // We mask off the sign bit and mark subnormals having a significand
        // less than 2¹²⁸ in least bit of the representations high
        // u128. This allows to use only that part for the handling of
        // special cases.
        let abs_bits_sticky_x = abs_bits_sticky(&abs_bits_x);
        let abs_bits_sticky_y = abs_bits_sticky(&abs_bits_y);
        if (abs_bits_sticky_x, abs_bits_sticky_y).any_special() {
            if max(abs_bits_sticky_x, abs_bits_sticky_y) > HI_EXP_MASK {
                // Atleast one operand is NAN.
                return f256::NAN;
            }
            if abs_bits_sticky_x == 0_u128 {
                return if abs_bits_sticky_y == 0 {
                    // Both operands are zero.
                    f256::ZERO
                } else {
                    // other = 0, self != 0 => ±½π
                    let mut res = FRAC_PI_2;
                    res.bits.hi.0 |= sign_bits_hi(&self);
                    res
                };
            }
            if abs_bits_sticky_y == 0_u128 {
                // self = 0, other > 0 => 0
                // self = 0, other < 0 => π
                return [f256::ZERO, PI][other.sign() as usize];
            }
            // Both operands are infinite.
            return match (self.sign(), other.sign()) {
                (0, 0) => FRAC_PI_4,
                // TODO: replace by constant FRAC_3_PI_2
                (0, 1) => &PI - &FRAC_PI_4,
                (1, 0) => -FRAC_PI_4,
                _ => &FRAC_PI_4 - &PI,
            };
        }

        // Both operands are finite and non-zero.

        let sign_q = (self.sign() + other.sign()) % 2;
        let mut atan = if abs_bits_y < abs_bits_x {
            let mut q = Float256::from(self);
            q /= &Float256::from(other);
            Float256::from(&approx_atan(&FP492::from(&q)))
        } else if abs_bits_y > abs_bits_x {
            let mut q = Float256::from(other);
            q /= &Float256::from(self);
            [Float256::FRAC_PI_2, -Float256::FRAC_PI_2][sign_q as usize]
                - Float256::from(&approx_atan(&FP492::from(&q)))
        } else {
            [Float256::FRAC_PI_2, -Float256::FRAC_PI_2][sign_q as usize]
        };
        match (self.sign(), other.sign()) {
            (0, 1) => {
                atan += &Float256::PI;
            }
            (1, 1) => {
                atan -= &Float256::PI;
            }
            _ => {}
        }
        Self::from(&atan)
    }
}

#[cfg(test)]
mod atan_tests {
    use core::{ops::Neg, str::FromStr};

    use super::*;
    use crate::{
        consts::{FRAC_1_PI, FRAC_PI_3, FRAC_PI_4, FRAC_PI_6},
        EPSILON,
    };

    #[test]
    fn calc_small_cutoff() {
        let mut lf = f256::from(1e-36_f64);
        let mut uf = f256::from(1e-35_f64);
        assert_eq!(lf, lf.atan());
        assert_ne!(uf, uf.atan());
        let mut f = (lf + uf) / f256::TWO;
        while lf < f && f < uf {
            if f == f.atan() {
                lf = f;
            } else {
                uf = f;
            }
            f = (lf + uf) / f256::TWO;
        }
        // println!("\n{lf:?}\n{:?}", lf.atan());
        // println!("\n{f:?}\n{:?}", f.atan());
        // println!("\n{uf:?}\n{:?}", uf.atan());
        // println!("\n// {f:e}");
        // println!("const SMALL_CUT_OFF: f256 = f256 {{");
        // println!("    bits: U256::new(");
        // println!(
        //     "        0x{:032x},\n        0x{:032x},\n    ),\n}};",
        //     f.bits.hi.0, f.bits.lo.0
        // );

        assert_eq!(f, f.atan());
        assert_eq!(f, SMALL_CUT_OFF);
        let g = f + f.ulp();
        assert_ne!(g, g.atan());
    }

    #[test]
    fn test_atan_inf() {
        assert_eq!(f256::INFINITY.atan(), FRAC_PI_2);
    }

    #[test]
    fn test_atan_large_cutoff() {
        let f = f256 {
            bits: LARGE_CUT_OFF,
        };
        assert_eq!(f.atan(), FRAC_PI_2);
    }

    #[test]
    fn test_atan_zero() {
        assert_eq!(f256::ZERO.atan(), f256::ZERO);
        assert_eq!(f256::NEG_ZERO.atan(), f256::ZERO);
    }

    #[test]
    fn test_atan_one() {
        assert_eq!(f256::ONE.atan(), FRAC_PI_4);
        assert_eq!(f256::NEG_ONE.atan(), -FRAC_PI_4);
    }

    #[test]
    fn test_atan_sqrt_3() {
        let t = f256::from(3);
        let mut f = t.sqrt();
        // arctan √3 = ⅓π
        assert_eq!(f.atan(), FRAC_PI_3);
        assert_eq!(f.neg().atan(), -FRAC_PI_3);
        // arctan ⅓√3 = π/6
        f /= t;
        assert_eq!(f.atan(), FRAC_PI_6);
        assert_eq!(f.neg().atan(), -FRAC_PI_6);
    }

    #[test]
    fn test_atan_frac_1_pi() {
        let f1 = FRAC_1_PI.atan();
        let f2 = f256::ONE.atan2(&PI);
        let d = f1 - f2;
        assert!(d.abs() <= EPSILON);
    }

    #[test]
    fn test_atan_frac_1_over_256() {
        let x = f256::from(256).recip();
        let ax = f256::from_str(
            "3.90623013196697182762866531142438714035749011520285621521309514901134417e-3")
            .unwrap();
        let r = x.atan();
        assert_eq!(ax, r);
    }

    #[test]
    fn test_atan_frac_pi_2() {
        let s = "1.00388482185388721414842394491713228829210446059487057472971282410801519";
        let a = f256::from_str(s).unwrap();
        let f1 = FRAC_PI_2.atan();
        assert_eq!(f1, a, "{} != {}", f1, a);
        let f2 = PI.atan2(&f256::TWO);
        assert_eq!(f1, f2, "{} != {}", f1, f2);
    }

    #[test]
    fn test_atan_frac_5_pi_4() {
        let s = "1.32144796778372235539166569069508390109061014033053361477468861418765787";
        let a = f256::from_str(s).unwrap();
        let f = PI + FRAC_PI_4;
        assert_eq!(f.atan(), a);
        let f = f256::TEN * PI;
        assert_eq!(f.atan2(&f256::from(8_f64)), a);
    }

    #[test]
    fn test_atan_frac_51043_7() {
        let s = "1.570659187521027203661619536335073835579283228441242208112611672132902725";
        let a = f256::from_str(s).unwrap();
        let n = f256::from(51043);
        let d = f256::from(7);
        let f = n / d;
        assert_eq!(n.atan2(&d), a);
    }
}
