// ---------------------------------------------------------------------------
// Copyright:   (c) 2024 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use super::u256;
use crate::{consts::FRAC_PI_2, f256};

impl f256 {
    /// Simultaneously computes the sine and cosine of the number x
    /// (in radians).
    ///
    /// Returns (sin(x), cos(x)).
    pub fn sin_cos(&self) -> (Self, Self) {
        (self.sin(), self.cos())
    }
}

#[cfg(test)]
mod sin_cos_tests {
    use super::*;
    use crate::{
        consts::{FRAC_PI_3, FRAC_PI_4, FRAC_PI_6},
        ONE_HALF,
    };

    //noinspection DuplicatedCode
    #[test]
    fn test_frac_pi_2_multiples() {
        const EXACT: [(f256, f256); 4] = [
            (f256::ZERO, f256::ONE),
            (f256::ONE, f256::ZERO),
            (f256::ZERO, f256::NEG_ONE),
            (f256::NEG_ONE, f256::ZERO),
        ];
        for i in 0_u32..=4_u32 {
            let eps = f256::EPSILON.mul_pow2(i as i32 / 4);
            let f = f256::from(i) * FRAC_PI_2;
            let (sin, cos) = f.sin_cos();
            let (exact_sin, exact_cos) = EXACT[(i % 4) as usize];
            if exact_sin.eq_zero() {
                let d = (exact_sin - sin).abs();
                assert!(d < eps, "{d:?} >= {eps:?}");
            } else {
                assert_eq!(exact_sin, sin);
            }
            if exact_cos.eq_zero() {
                let d = (exact_cos - cos).abs();
                assert!(d < eps, "{d:?} >= {eps:?}");
            } else {
                assert_eq!(exact_cos, cos);
            }
        }
    }

    #[test]
    fn test_signs() {
        let p = FRAC_PI_4;
        for i in 0..9 {
            let f = f256::from(i) * FRAC_PI_2 + p;
            let (sin, cos) = f.sin_cos();
            let quadrant = i % 4;
            match quadrant {
                0 => {
                    assert!(sin.is_sign_positive());
                    assert!(cos.is_sign_positive());
                }
                1 => {
                    assert!(sin.is_sign_positive());
                    assert!(cos.is_sign_negative());
                }
                2 => {
                    assert!(sin.is_sign_negative());
                    assert!(cos.is_sign_negative());
                }
                3 => {
                    assert!(sin.is_sign_negative());
                    assert!(cos.is_sign_positive());
                }
                _ => unreachable!(),
            }
        }
    }

    #[test]
    fn test_frac_pi_4() {
        // sin(45°) = cos(45°)
        let f = FRAC_PI_4;
        let (sin, cos) = f.sin_cos();
        assert!((sin - cos).abs() <= sin.ulp());
    }

    #[test]
    fn test_frac_pi_3_and_frac_pi_6() {
        // sin(30°) = 0.5
        let sin = FRAC_PI_6.sin();
        assert_eq!(sin, ONE_HALF);
        // cos(60°) = 0.5
        let cos = FRAC_PI_3.cos();
        assert_eq!(cos, ONE_HALF);
        // sin(60°) = cos(30°)
        let sin = FRAC_PI_3.sin();
        let cos = FRAC_PI_6.cos();
        assert_eq!(sin, cos);
    }

    #[test]
    fn test_some_lt_2pi() {
        let f = f256::from_sign_exp_signif(
            0,
            -261,
            (
                0x0000100410f1f3ab981fc5a9fd008e6e,
                0x6ba97c4190d331836d7fd41d2009cdf8,
            ),
        );
        let sin_f = f256::from_sign_exp_signif(
            0,
            -261,
            (
                0x0000100410f1f3ab977498bfffb5d0d5,
                0xd4afb6f12a8836a249b17fbeb758fa8e,
            ),
        );
        assert_eq!(f.sin(), sin_f);
        let f = f256::from_sign_exp_signif(
            0,
            -235,
            (
                0x000019412990c230cfe83e598062a70f,
                0x2e55ff0ee1b47200750f278655e459cc,
            ),
        );
        let sin_f = f256::from_sign_exp_signif(
            1,
            -243,
            (
                0x00001f2ded8c6c188d6563858850bd6f,
                0xc0dd632c3566aef3b1af2c6bd810e0fe,
            ),
        );
        assert_eq!(f.sin(), sin_f);
        let f = f256::from_sign_exp_signif(
            0,
            -230,
            (
                0x000001709d10d3e7eab960be165f5516,
                0xe8df7f75d98f0fa868f6d4ae6add8617,
            ),
        );
        let sin_f = f256::from_sign_exp_signif(
            1,
            -237,
            (
                0x00000fffffffffffffffffffffffffff,
                0xfffffffffffffffffffffffffffffffd,
            ),
        );
        assert_eq!(f.sin(), sin_f, "{f}\n{}\n{}", f.sin(), sin_f);
    }

    #[test]
    fn test_some_gt_2pi() {
        // 451072.762503992264821001752482581001682512026226517387166060002390623476
        let f = f256::from_sign_exp_signif(
            0,
            -218,
            (
                0x00001b88030ccdd8b7632adb619b1f1f,
                0x0e1d0adefedbcedd03c621b5967e9c1d,
            ),
        );
        // 0.249623167582990240382008743809080852087294584792829298735057191909919803
        let sin_f = f256::from_sign_exp_signif(
            0,
            -239,
            (
                0x00001ff3a6e68be32dc92aa6c6930521,
                0x192865a8b728d2d42fcb7319995fc955,
            ),
        );
        println!("{f}\n{sin_f}");
        assert_eq!(f.sin(), sin_f);
    }

    #[test]
    fn test_neg_values() {
        for f in [f256::MIN, -FRAC_PI_3, f256::NEG_ONE] {
            assert_eq!(f.sin(), -f.abs().sin());
            assert_eq!(f.cos(), f.abs().cos());
        }
    }

    #[test]
    fn test_continuity_near_zero() {
        let c = f256::from(1e-36);
        let d = c.ulp();
        let mut f = c;
        let mut g = f;
        for i in 0..1000 {
            g += d;
            assert!(f < g);
            assert!(
                f.sin() <= g.sin(),
                "    f: {}\nsin f: {}\n    g: {}\nsin g: {}",
                f,
                f.sin(),
                g,
                g.sin()
            );
            assert!(
                f.cos() >= g.cos(),
                "    f: {}\ncos f: {}\n    g: {}\ncos g: {}",
                f,
                f.cos(),
                g,
                g.cos()
            );
            f = g;
        }
    }

    //noinspection DuplicatedCode
    #[test]
    fn test_continuity_near_one() {
        let c = f256::ONE;
        let d = f256::EPSILON;
        let mut f = c;
        let mut g = f;
        for i in 0..10 {
            g += d;
            assert!(f < g);
            assert!(f.sin() <= g.sin());
            assert!(f.cos() >= g.cos());
            f = g;
        }
    }

    //noinspection DuplicatedCode
    #[test]
    fn test_continuity_near_pi_half() {
        let c = FRAC_PI_2;
        let d = f256::from(1.5e-36);
        let mut f = c + d;
        let mut g = f;
        for i in 0..10 {
            g += d;
            assert!(f < g);
            assert!(f.sin() >= g.sin());
            assert!(f.cos() <= g.cos());
            f = g;
        }
        let mut f = c;
        let mut g = f;
        for i in 0..10 {
            g -= d;
            assert!(f > g);
            assert!(f.sin() >= g.sin());
            assert!(f.cos() <= g.cos());
            f = g;
        }
    }

    //noinspection DuplicatedCode
    #[test]
    fn test_continuity_near_three() {
        let c = f256::from(3);
        let d = f256::EPSILON * f256::TWO;
        let mut f = c;
        let mut g = f;
        for i in 0..10 {
            g += d;
            assert!(f < g);
            assert!(f.sin() >= g.sin());
            assert!(f.cos() <= g.cos());
            f = g;
        }
    }

    #[test]
    fn test_small_value() {
        let f = f256::from_sign_exp_signif(
            0,
            -268,
            (
                511713792246730580583350097904921,
                338234285556250629981528767706881153057,
            ),
        );
        let sin = f256::from_sign_exp_signif(
            0,
            -268,
            (
                511713792246730580571854506161847,
                105438061704425261882515718706001931297,
            ),
        );
        assert_eq!(f.sin(), sin);
    }

    #[test]
    fn test_nearest_to_pi_over_2() {
        let f = FRAC_PI_2 - FRAC_PI_2.ulp();
        assert_ne!(f, FRAC_PI_2);
        assert_eq!(f.sin(), f256::ONE);
        assert_eq!((f - FRAC_PI_2).cos(), f256::ONE);
        let f = FRAC_PI_2 + FRAC_PI_2.ulp();
        assert_ne!(f, FRAC_PI_2);
        assert_eq!(f.sin(), f256::ONE);
        assert_eq!((f - FRAC_PI_2).cos(), f256::ONE);
    }

    // f: 140844820278614289426057198173335166586563126037009815346672127671657710 * 2^185461
    // ε: 5.769198204535869190785720230896528973489817286545990660946235357113661705e-77
    // -log₂(ε): 253.26
}
