// ---------------------------------------------------------------------------
// Copyright:   (c) 2022 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

#[cfg(test)]
mod sub_tests {
    use core::cmp::Ordering;

    use f256::f256;

    #[test]
    fn test_nan() {
        assert!((f256::NAN - f256::ONE).is_nan());
        assert!((f256::ONE - f256::NAN).is_nan());
        assert!((f256::NAN - f256::NAN).is_nan());
        assert!((f256::NAN - f256::INFINITY).is_nan());
        assert!((f256::INFINITY - f256::NAN).is_nan());
        assert!((f256::NAN - f256::NEG_INFINITY).is_nan());
        assert!((f256::NEG_INFINITY - f256::NAN).is_nan());
    }

    #[test]
    fn test_inf() {
        assert_eq!(f256::INFINITY - f256::NEG_INFINITY, f256::INFINITY);
        assert_eq!(f256::INFINITY - f256::ONE, f256::INFINITY);
        assert_eq!(f256::ONE - f256::INFINITY, f256::NEG_INFINITY);
        assert_eq!(f256::NEG_INFINITY - f256::INFINITY, f256::NEG_INFINITY);
        assert_eq!(f256::NEG_INFINITY - f256::ONE, f256::NEG_INFINITY);
        assert_eq!(f256::ONE - f256::NEG_INFINITY, f256::INFINITY);
        assert!((f256::INFINITY - f256::INFINITY).is_nan());
        assert!((f256::NEG_INFINITY - f256::NEG_INFINITY).is_nan());
    }

    #[test]
    fn test_zero() {
        // Because the normal cmp treats 0 == -0, we have to use total_cmp.
        assert_eq!(
            (f256::ZERO - f256::ZERO).total_cmp(&f256::ZERO),
            Ordering::Equal
        );
        assert_eq!(
            (f256::ZERO - f256::NEG_ZERO).total_cmp(&f256::ZERO),
            Ordering::Equal
        );
        assert_eq!(
            (f256::NEG_ZERO - f256::ZERO).total_cmp(&f256::NEG_ZERO),
            Ordering::Equal
        );
        assert_eq!(
            (f256::NEG_ZERO - f256::NEG_ZERO).total_cmp(&f256::ZERO),
            Ordering::Equal
        );
        assert_eq!(f256::ONE - f256::ZERO, f256::ONE);
        assert_eq!(f256::ZERO - f256::ONE, f256::NEG_ONE);
        assert_eq!(f256::ONE - f256::NEG_ZERO, f256::ONE);
        assert_eq!(f256::NEG_ZERO - f256::ONE, f256::NEG_ONE);
    }

    #[test]
    fn test_normal() {
        assert_eq!(f256::ONE - f256::ONE, f256::ZERO);
        assert_eq!(f256::ONE - f256::TWO, f256::NEG_ONE);
        assert_eq!(f256::from(4.0) - f256::TWO, f256::TWO);
        assert_eq!(f256::from(7.0) - f256::from(3.5), f256::from(3.5));
        assert_eq!(f256::MAX - f256::MAX, f256::ZERO);
        assert_eq!(f256::MIN - f256::MIN, f256::ZERO);
        assert_eq!(f256::MAX - f256::EPSILON, f256::MAX);
        assert_eq!(f256::MIN - f256::EPSILON, f256::MIN);
        assert_eq!(f256::MAX - f256::MIN_GT_ZERO, f256::MAX);
        assert_eq!(f256::MIN - f256::MIN_GT_ZERO, f256::MIN);
    }

    #[test]
    fn test_normal_small_diff() {
        let x = f256::from_sign_exp_signif(
            0,
            -183,
            (608472288109550112718437538580480, 7005),
        );
        assert!(x.is_normal());
        let y = f256::from_sign_exp_signif(
            0,
            -183,
            (608472288109550112718437538580480, 7001),
        );
        assert!(y.is_normal());
        let z = f256::from_sign_exp_signif(0, -183, (0, 4));
        assert!(z.is_normal());
        assert_eq!(x - y, z);
        assert_eq!(y - x, -z);
    }

    #[test]
    fn test_normal_sub_normal_giving_subnormal() {
        let x = f256::from_sign_exp_signif(
            0,
            -262376,
            (324518553658426726783156020576256, 12009),
        );
        assert!(x.is_normal());
        let y = f256::from_sign_exp_signif(
            0,
            -262377,
            (567907468902246771870523036008448, 118321),
        );
        assert!(y.is_normal());
        let z = f256::from_sign_exp_signif(
            0,
            -262377,
            (
                81129638414606681695789005144063,
                340282366920938463463374607431768117153,
            ),
        );
        assert!(z.is_subnormal());
        assert_eq!(x - y, z);
        assert_eq!(y - x, -z);
    }

    #[test]
    fn test_normal_sub_normal_same_exp_giving_subnormal() {
        let x = f256::from_sign_exp_signif(
            0,
            -262376,
            (608472288109550112718437538580480, 7001),
        );
        assert!(x.is_normal());
        let y = f256::from_sign_exp_signif(
            0,
            -262376,
            (608472288109550112718437538580480, 7000),
        );
        assert!(y.is_normal());
        let z = f256::from_sign_exp_signif(0, -262376, (0, 1));
        assert!(z.is_subnormal());
        assert_eq!(x - y, z);
        assert_eq!(y - x, -z);
    }

    #[test]
    fn test_subnormal_sub_subnormal() {
        let x = f256::from_sign_exp_signif(0, -262378, (37538580480, 7031));
        assert!(x.is_subnormal());
        let y = f256::from_sign_exp_signif(0, -262378, (37538580480, 52));
        assert!(y.is_subnormal());
        let z = f256::from_sign_exp_signif(0, -262378, (0, 6979));
        assert!(z.is_subnormal());
        assert_eq!(x - y, z);
        assert_eq!(y - x, -z);
    }

    #[test]
    fn test_min_gt_zero() {
        assert_eq!(f256::MAX - f256::MIN_GT_ZERO, f256::MAX);
        assert_eq!(f256::MIN - f256::MIN_GT_ZERO, f256::MIN);
        assert_eq!(f256::MIN_GT_ZERO - f256::MAX, f256::MIN);
        assert_eq!(f256::MIN_GT_ZERO - f256::MIN, f256::MAX);
        assert_eq!(f256::ONE - f256::MIN_GT_ZERO, f256::ONE);
        assert_eq!(f256::MIN_GT_ZERO - f256::ONE, f256::NEG_ONE);
        assert_eq!(f256::MIN_GT_ZERO - f256::MIN_GT_ZERO, f256::ZERO);
    }

    #[test]
    fn test_overflow() {
        assert_eq!(f256::MIN - f256::MAX, f256::NEG_INFINITY);
        assert_eq!(f256::MIN - f256::ONE, f256::MIN);
    }
}
