// ---------------------------------------------------------------------------
// Copyright:   (c) 2022 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

#[cfg(test)]
mod add_tests {
    use core::cmp::Ordering;

    use f256::f256;

    #[test]
    fn test_nan() {
        assert!((f256::NAN + f256::ONE).is_nan());
        assert!((f256::ONE + f256::NAN).is_nan());
        assert!((f256::NAN + f256::NAN).is_nan());
        assert!((f256::NAN + f256::INFINITY).is_nan());
        assert!((f256::INFINITY + f256::NAN).is_nan());
        assert!((f256::NAN + f256::NEG_INFINITY).is_nan());
        assert!((f256::NEG_INFINITY + f256::NAN).is_nan());
    }

    #[test]
    fn test_inf() {
        assert_eq!(f256::INFINITY + f256::INFINITY, f256::INFINITY);
        assert_eq!(f256::INFINITY + f256::ONE, f256::INFINITY);
        assert_eq!(f256::ONE + f256::INFINITY, f256::INFINITY);
        assert_eq!(f256::NEG_INFINITY + f256::NEG_INFINITY, f256::NEG_INFINITY);
        assert_eq!(f256::NEG_INFINITY + f256::ONE, f256::NEG_INFINITY);
        assert_eq!(f256::ONE + f256::NEG_INFINITY, f256::NEG_INFINITY);
        assert!((f256::INFINITY + f256::NEG_INFINITY).is_nan());
        assert!((f256::NEG_INFINITY + f256::INFINITY).is_nan());
    }

    #[test]
    fn test_zero() {
        // Because the normal cmp treats 0 == -0, we have to use total_cmp.
        assert_eq!(
            (f256::ZERO + f256::ZERO).total_cmp(&f256::ZERO),
            Ordering::Equal
        );
        assert_eq!(
            (f256::ZERO + f256::NEG_ZERO).total_cmp(&f256::ZERO),
            Ordering::Equal
        );
        assert_eq!(
            (f256::NEG_ZERO + f256::ZERO).total_cmp(&f256::ZERO),
            Ordering::Equal
        );
        assert_eq!(
            (f256::NEG_ZERO + f256::NEG_ZERO).total_cmp(&f256::NEG_ZERO),
            Ordering::Equal
        );
        assert_eq!(f256::ONE + f256::ZERO, f256::ONE);
        assert_eq!(f256::ZERO + f256::ONE, f256::ONE);
        assert_eq!(f256::ONE + f256::NEG_ZERO, f256::ONE);
        assert_eq!(f256::NEG_ZERO + f256::ONE, f256::ONE);
    }

    #[test]
    fn test_normal() {
        assert_eq!(f256::ONE + f256::ONE, f256::TWO);
        assert_eq!(f256::ONE + f256::NEG_ONE, f256::ZERO);
        assert_eq!(f256::TWO + f256::TWO, f256::from(4.0));
        assert_eq!(f256::from(3.5) + f256::from(3.5), f256::from(7.0));
        assert_eq!(f256::from(3.5) + f256::from(-3.5), f256::ZERO);
        assert_eq!(f256::from(-3.5) + f256::from(-3.5), f256::from(-7.0));
        assert_eq!(f256::MAX + f256::MIN, f256::ZERO);
        assert_eq!(f256::MIN + f256::MAX, f256::ZERO);
        assert_eq!(f256::MAX + f256::EPSILON, f256::MAX);
        assert_eq!(f256::MIN + f256::EPSILON, f256::MIN);
        assert_eq!(f256::MAX + f256::MIN_GT_ZERO, f256::MAX);
        assert_eq!(f256::MIN + f256::MIN_GT_ZERO, f256::MIN);
    }

    #[test]
    fn test_subnormal_add_subnormal_giving_subnormal() {
        let x = f256::from_sign_exp_signif(0, -262378, (37538580480, 352));
        assert!(x.is_subnormal());
        let y = f256::from_sign_exp_signif(0, -262378, (17, 65003));
        assert!(y.is_subnormal());
        let z = f256::from_sign_exp_signif(0, -262378, (37538580497, 65355));
        assert!(z.is_subnormal());
        assert_eq!(x + y, z);
        assert_eq!(y + x, z);
    }

    #[test]
    fn test_subnormal_add_subnormal_giving_normal() {
        let x = f256::from_sign_exp_signif(
            0,
            -262378,
            (
                324518553658426726783156020576255,
                340282366920938463463374607431768199447,
            ),
        );
        assert!(x.is_subnormal());
        let y = f256::from_sign_exp_signif(
            0,
            -262378,
            (
                2535301200456458802993406410751,
                340282366920938463463374607431768211363,
            ),
        );
        assert!(y.is_subnormal());
        let z = f256::from_sign_exp_signif(
            0,
            -262377,
            (
                163526927429441592793074713493503,
                340282366920938463463374607431768205405,
            ),
        );
        assert!(z.is_normal());
        assert_eq!(x + y, z);
        assert_eq!(y + x, z);
    }

    #[test]
    fn test_normal_add_subnormal() {
        let x = f256::from_sign_exp_signif(
            0,
            -262376,
            (
                649037107316853453566312041152511,
                340282366920938463463374607431768187439,
            ),
        );
        assert!(x.is_normal());
        let y = f256::from_sign_exp_signif(
            0,
            -262378,
            (
                2535301200456458802993406410751,
                340282366920938463463374607431768211363,
            ),
        );
        assert!(y.is_subnormal());
        let z = f256::from_sign_exp_signif(
            0,
            -262374,
            (
                162338504991727627729171554238463,
                340282366920938463463374607431768205449,
            ),
        );
        assert!(z.is_normal());
        assert_eq!(x + y, z);
        assert_eq!(y + x, z);
    }

    #[test]
    fn test_min_gt_zero() {
        assert_eq!(f256::MAX + f256::MIN_GT_ZERO, f256::MAX);
        assert_eq!(f256::MIN + f256::MIN_GT_ZERO, f256::MIN);
        assert_eq!(f256::MIN_GT_ZERO + f256::MAX, f256::MAX);
        assert_eq!(f256::MIN_GT_ZERO + f256::MIN, f256::MIN);
        assert_eq!(f256::ONE + f256::MIN_GT_ZERO, f256::ONE);
        assert_eq!(f256::MIN_GT_ZERO + f256::ONE, f256::ONE);
    }

    #[test]
    fn test_overflow() {
        assert_eq!(f256::MAX + f256::MAX, f256::INFINITY);
        assert_eq!(f256::MAX + f256::ONE, f256::MAX);
    }
}
