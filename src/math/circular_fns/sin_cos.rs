// ---------------------------------------------------------------------------
// Copyright:   (c) 2024 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use super::u256;
use crate::{
    consts::FRAC_PI_2,
    f256,
    math::circular_fns::{
        approx_cos::approx_cos, approx_sin::approx_sin,
        approx_sin_cos::approx_sin_cos, reduce::reduce,
    },
    HI_ABS_MASK,
};

impl f256 {
    /// Simultaneously computes the sine and cosine of the number x
    /// (in radians).
    ///
    /// Returns (sin(x), cos(x)).
    pub fn sin_cos(&self) -> (Self, Self) {
        if self.is_special() {
            // x is NAN or infinite => sine x and cosine x are NAN
            if (self.bits.hi & HI_ABS_MASK) > f256::MAX.bits.hi {
                return (f256::NAN, f256::NAN);
            }
            // x = 0 => sine x = 0 and cosine x = 1
            return (f256::ZERO, f256::ONE);
        }
        // Calculate ⌈|x|/½π⌋ % 4 and |x| % ½π.
        let (quadrant, fx) = reduce(&self.abs());
        // Approximate sine and cosine
        let (sin, cos) = approx_sin_cos(&fx);
        // Map result according to quadrant and sign
        match (quadrant, self.sign()) {
            (0, 0) => (Self::from(&sin), Self::from(&cos)),
            (0, 1) => (-Self::from(&sin), Self::from(&cos)),
            (1, 0) => (Self::from(&cos), -Self::from(&sin)),
            (1, 1) => (-Self::from(&cos), -Self::from(&sin)),
            (2, 0) => (-Self::from(&sin), -Self::from(&cos)),
            (2, 1) => (Self::from(&sin), -Self::from(&cos)),
            (3, 0) => (-Self::from(&cos), Self::from(&sin)),
            (3, 1) => (Self::from(&cos), Self::from(&sin)),
            _ => unreachable!(),
        }
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
