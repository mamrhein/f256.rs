// ---------------------------------------------------------------------------
// Copyright:   (c) 2022 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

extern crate core;
extern crate f256 as f;

#[cfg(test)]
mod div_euclid_tests {
    use f::consts::{E, PI};
    use f::f256;

    #[test]
    fn test_nan() {
        assert!(f256::NAN.div_euclid(f256::NAN).is_nan());
        for f in [
            f256::NEG_INFINITY,
            f256::MIN,
            f256::NEG_ZERO,
            f256::ZERO,
            f256::MAX,
            f256::INFINITY,
        ] {
            assert!(f.div_euclid(f256::NAN).is_nan());
            assert!(f256::NAN.div_euclid(f).is_nan());
        }
    }

    #[test]
    fn test_zero() {
        // Test with zero as divisor
        assert!(f256::ZERO.div_euclid(f256::ZERO).is_nan());
        assert!(f256::NEG_ZERO.div_euclid(f256::ZERO).is_nan());
        assert_eq!(f256::ONE.div_euclid(f256::ZERO), f256::INFINITY);
        assert_eq!(f256::NEG_ONE.div_euclid(f256::ZERO), f256::NEG_INFINITY);
        assert_eq!(f256::INFINITY.div_euclid(f256::ZERO), f256::INFINITY);
        assert_eq!(
            f256::NEG_INFINITY.div_euclid(f256::ZERO),
            f256::NEG_INFINITY
        );
        // Test with negative zero as divisor
        assert!(f256::ZERO.div_euclid(f256::NEG_ZERO).is_nan());
        assert!(f256::NEG_ZERO.div_euclid(f256::NEG_ZERO).is_nan());
        assert_eq!(f256::ONE.div_euclid(f256::NEG_ZERO), f256::NEG_INFINITY);
        assert_eq!(f256::NEG_ONE.div_euclid(f256::NEG_ZERO), f256::INFINITY);
        assert_eq!(
            f256::INFINITY.div_euclid(f256::NEG_ZERO),
            f256::NEG_INFINITY
        );
        assert_eq!(
            f256::NEG_INFINITY.div_euclid(f256::NEG_ZERO),
            f256::INFINITY
        );
        // Test with zero as divident
        assert_eq!(f256::ZERO.div_euclid(f256::ONE), f256::ZERO);
        assert_eq!(f256::ZERO.div_euclid(f256::NEG_ONE), f256::NEG_ZERO);
        assert_eq!(f256::ZERO.div_euclid(f256::INFINITY), f256::ZERO);
        assert_eq!(f256::ZERO.div_euclid(f256::NEG_INFINITY), f256::NEG_ZERO);
        // Test with negative zero as divident
        assert_eq!(f256::NEG_ZERO.div_euclid(f256::ONE), f256::NEG_ZERO);
        assert_eq!(f256::NEG_ZERO.div_euclid(f256::NEG_ONE), f256::ZERO);
        assert_eq!(f256::NEG_ZERO.div_euclid(f256::INFINITY), f256::NEG_ZERO);
        assert_eq!(f256::NEG_ZERO.div_euclid(f256::NEG_INFINITY), f256::ZERO);
    }

    #[test]
    fn test_infinity() {
        // Test with infinity as divisor
        assert!(f256::INFINITY.div_euclid(f256::INFINITY).is_nan());
        assert!(f256::NEG_INFINITY.div_euclid(f256::INFINITY).is_nan());
        assert_eq!(f256::ZERO.div_euclid(f256::INFINITY), f256::ZERO);
        assert_eq!(f256::NEG_ZERO.div_euclid(f256::INFINITY), f256::NEG_ZERO);
        assert_eq!(f256::ONE.div_euclid(f256::INFINITY), f256::ZERO);
        assert_eq!(f256::NEG_ONE.div_euclid(f256::INFINITY), f256::NEG_ZERO);
        // Test with negative infinity as divisor
        assert!(f256::INFINITY.div_euclid(f256::NEG_INFINITY).is_nan());
        assert!(f256::NEG_INFINITY.div_euclid(f256::NEG_INFINITY).is_nan());
        assert_eq!(f256::ZERO.div_euclid(f256::NEG_INFINITY), f256::NEG_ZERO);
        assert_eq!(f256::NEG_ZERO.div_euclid(f256::NEG_INFINITY), f256::ZERO);
        assert_eq!(f256::ONE.div_euclid(f256::NEG_INFINITY), f256::NEG_ZERO);
        assert_eq!(f256::NEG_ONE.div_euclid(f256::NEG_INFINITY), f256::ZERO);
        // Test with infinity as dividend
        assert_eq!(f256::INFINITY.div_euclid(f256::TWO), f256::INFINITY);
        assert_eq!(f256::INFINITY.div_euclid(-f256::TEN), f256::NEG_INFINITY);
        // Test with negative infinity as dividend
        assert_eq!(
            f256::NEG_INFINITY.div_euclid(f256::TWO),
            f256::NEG_INFINITY
        );
        assert_eq!(f256::NEG_INFINITY.div_euclid(-f256::TEN), f256::INFINITY);
    }

    #[test]
    fn test_div_euclid_edge_cases() {
        // Test with subnormal numbers
        let subnormal = f256::MIN_POSITIVE;
        assert_eq!(f256::ZERO.div_euclid(subnormal), f256::ZERO);
        assert_eq!(subnormal.div_euclid(f256::ZERO), f256::INFINITY);
        assert_eq!(subnormal.div_euclid(f256::ONE), f256::ZERO);
        // Test with maximum and minimum values
        assert_eq!(f256::MAX.div_euclid(f256::ONE), f256::MAX);
        assert_eq!(f256::MIN.div_euclid(f256::ONE), f256::MIN);
        assert_eq!(f256::MAX.div_euclid(f256::MAX), f256::ONE);
        assert_eq!(f256::MIN.div_euclid(f256::MIN), f256::ONE);
        // Test with very small positive numbers
        let tiny = f256::EPSILON;
        assert_eq!(tiny.div_euclid(f256::ONE), f256::ZERO);
        assert_eq!(tiny.div_euclid(tiny), f256::ONE);
    }

    #[test]
    fn test_div_euclid_sign_combinations() {
        // Test all sign combinations
        let tests = vec![
            (f256::ONE, f256::ONE, f256::ONE),
            (f256::NEG_ONE, f256::ONE, f256::NEG_ONE),
            (f256::ONE, f256::NEG_ONE, f256::NEG_ONE),
            (f256::NEG_ONE, f256::NEG_ONE, f256::ONE),
            (f256::TWO, f256::TWO, f256::ONE),
            (-f256::TWO, f256::TWO, f256::NEG_ONE),
            (f256::TWO, -f256::TWO, f256::NEG_ONE),
            (-f256::TWO, -f256::TWO, f256::ONE),
        ];

        for (dividend, divisor, expected) in tests {
            assert_eq!(dividend.div_euclid(divisor), expected);
        }
    }

    #[test]
    fn test_div_euclid_abs_divident_less_than_abs_divisor() {
        for d in [f256::TEN, -f256::TEN] {
            for f in [
                -f256::MIN_POSITIVE,
                -f256::TWO,
                f256::ONE,
                f256::TEN.next_down(),
            ] {
                if f.signum() == d.signum() {
                    assert!(f.div_euclid(d).eq_zero())
                } else {
                    assert_eq!(f.div_euclid(d), f256::NEG_ONE)
                };
            }
        }
        assert!(E.div_euclid(PI).eq_zero());
    }

    #[test]
    fn test_div_euclid_normal() {
        let mut f = f256::from(7.5_f64);
        let d = f256::TWO;
        assert_eq!(f.div_euclid(d), f256::from(3));
        assert_eq!(f.div_euclid(d.square()), f256::ONE);
        f = -f;
        assert_eq!(f.div_euclid(d), f256::from(-4));
        assert_eq!(f.div_euclid(d.square()), -d);
    }
}

#[cfg(test)]
mod rem_euclid_tests {
    use f::consts::{E, PI};
    use f::f256;

    #[test]
    fn test_nan() {
        assert!(f256::NAN.rem_euclid(f256::NAN).is_nan());
        for f in [
            f256::NEG_INFINITY,
            f256::MIN,
            f256::NEG_ZERO,
            f256::ZERO,
            f256::MAX,
            f256::INFINITY,
        ] {
            assert!(f.rem_euclid(f256::NAN).is_nan());
            assert!(f256::NAN.rem_euclid(f).is_nan());
        }
    }

    #[test]
    fn test_zero() {
        // Test with zero as divisor
        for f in [
            f256::NEG_INFINITY,
            f256::MIN,
            f256::NEG_ONE,
            f256::NEG_ZERO,
            f256::ZERO,
            f256::MIN_GT_ZERO,
            f256::MIN_POSITIVE,
            f256::TWO,
            f256::MAX,
            f256::INFINITY,
        ] {
            assert!(f.rem_euclid(f256::ZERO).is_nan());
            assert!(f.rem_euclid(f256::NEG_ZERO).is_nan());
        }
        // Test with zero as divident
        assert_eq!(f256::ZERO.rem_euclid(f256::ONE), f256::ZERO);
        assert_eq!(f256::ZERO.rem_euclid(f256::NEG_ONE), f256::NEG_ZERO);
        assert!(f256::ZERO.rem_euclid(f256::INFINITY).is_nan());
        assert!(f256::ZERO.rem_euclid(f256::NEG_INFINITY).is_nan());
        // Test with negative zero as divident
        assert_eq!(f256::NEG_ZERO.rem_euclid(f256::ONE), f256::NEG_ZERO);
        assert_eq!(f256::NEG_ZERO.rem_euclid(f256::NEG_ONE), f256::ZERO);
        assert!(f256::NEG_ZERO.rem_euclid(f256::INFINITY).is_nan());
        assert!(f256::NEG_ZERO.rem_euclid(f256::NEG_INFINITY).is_nan());
    }

    #[test]
    fn test_infinity() {
        for f in [
            f256::NEG_INFINITY,
            f256::MIN,
            f256::NEG_ONE,
            f256::NEG_ZERO,
            f256::ZERO,
            f256::MIN_GT_ZERO,
            f256::MIN_POSITIVE,
            f256::TWO,
            f256::MAX,
            f256::INFINITY,
        ] {
            assert!(f.rem_euclid(f256::INFINITY).is_nan());
            assert!(f.rem_euclid(f256::NEG_INFINITY).is_nan());
            assert!(f256::INFINITY.rem_euclid(f).is_nan());
            assert!(f256::NEG_INFINITY.rem_euclid(f).is_nan());
        }
    }

    #[test]
    fn test_rem_euclid_edge_cases() {
        // Test with subnormal numbers
        let mut subnormal = f256::MIN_POSITIVE;
        assert_eq!(f256::ZERO.rem_euclid(subnormal), f256::ZERO);
        assert_eq!(subnormal.rem_euclid(f256::ONE), subnormal);
        // Edge case: rem == divisor
        subnormal = -subnormal;
        assert_eq!(subnormal.rem_euclid(f256::ONE), f256::ONE);
        // Test with maximum and minimum values
        assert_eq!(f256::MAX.rem_euclid(f256::TWO), f256::ZERO);
        assert_eq!(f256::MIN.rem_euclid(f256::TWO), f256::ZERO);
        assert_eq!(f256::MAX.rem_euclid(f256::MAX), f256::ZERO);
        assert_eq!(f256::MIN.rem_euclid(f256::MIN), f256::ZERO);
        // Test with very small positive numbers
        let mut tiny = f256::EPSILON.ulp();
        assert_eq!(tiny.rem_euclid(f256::ONE), tiny);
        assert_eq!(tiny.rem_euclid(tiny), f256::ZERO);
        // Edge case: rem == divisor
        tiny = -tiny;
        assert_eq!(tiny.rem_euclid(f256::ONE), f256::ONE);
    }

    #[test]
    fn test_rem_euclid_sign_combinations() {
        // Test all sign combinations
        let tests = vec![
            (f256::ONE, f256::ONE),
            (f256::NEG_ONE, f256::ONE),
            (f256::ONE, f256::NEG_ONE),
            (f256::NEG_ONE, f256::NEG_ONE),
            (f256::TWO, f256::TWO),
            (-f256::TWO, f256::TWO),
            (f256::TWO, -f256::TWO),
            (-f256::TWO, -f256::TWO),
        ];

        for (dividend, divisor) in tests {
            assert!(dividend.rem_euclid(divisor).eq_zero());
        }
    }

    #[test]
    fn test_rem_euclid_abs_divident_less_than_abs_divisor() {
        for d in [f256::TEN, -f256::TEN] {
            for f in [
                -f256::MIN_POSITIVE,
                -f256::TWO,
                f256::ONE,
                f256::TEN.next_down(),
            ] {
                if f.signum() == d.signum() {
                    assert_eq!(f.rem_euclid(d), f);
                } else {
                    assert_eq!(f.rem_euclid(d), f + d);
                }
            }
        }
        assert_eq!(E.rem_euclid(PI), E);
    }

    #[test]
    fn test_rem_euclid_normal() {
        let mut f = f256::from(7.5);
        let d = f256::from(3);
        assert_eq!(f.rem_euclid(d), f256::from(1.5));
        assert_eq!(f.rem_euclid(d.square()), f);
        f = -f;
        assert_eq!(f.rem_euclid(d), f256::from(1.5));
        assert_eq!(f.rem_euclid(d.square()), f256::from(1.5));
    }
}

#[cfg(test)]
mod div_rem_euclid_tests {
    use f::consts::{E, PI};
    use f::f256;

    #[test]
    fn test_div_euclid_consistency_with_rem_euclid() {
        // For any valid x and y (y != 0):
        // x == (x.div_euclid(y) * y) + x.rem_euclid(y)
        let tests = vec![
            (f256::ONE, f256::TWO),
            (f256::NEG_ONE, f256::TWO),
            (f256::TWO, f256::ONE),
            (-f256::TWO, f256::ONE),
            (PI, E),
            (E, PI),
            (f256::MAX, f256::TWO),
            (f256::MIN, f256::TWO),
        ];

        for (dividend, divisor) in tests {
            let quotient = dividend.div_euclid(divisor);
            let remainder = dividend.rem_euclid(divisor);
            let result = quotient * divisor + remainder;

            // Due to floating point precision, we check if the difference is within epsilon
            let diff = (result - dividend).abs();
            assert!(diff <= f256::EPSILON || diff.eq_zero());
        }
    }
}
