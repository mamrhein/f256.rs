// ---------------------------------------------------------------------------
// Copyright:   (c) 2022 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

#[cfg(test)]
mod rem_tests {
    use std::str::FromStr;

    use f256::f256;

    #[test]
    fn test_nan() {
        assert!((f256::NAN % f256::ONE).is_nan());
        assert!((f256::ONE % f256::NAN).is_nan());
        assert!((f256::NAN % f256::NAN).is_nan());
        assert!((f256::NAN % f256::INFINITY).is_nan());
        assert!((f256::INFINITY % f256::NAN).is_nan());
        assert!((f256::NAN % f256::NEG_INFINITY).is_nan());
        assert!((f256::NEG_INFINITY % f256::NAN).is_nan());
    }

    #[test]
    fn test_inf() {
        assert!((f256::INFINITY % f256::ONE).is_nan());
        assert_eq!(f256::ONE % f256::INFINITY, f256::ONE);
        assert!((f256::NEG_INFINITY % f256::ONE).is_nan());
        assert_eq!(f256::ONE % f256::NEG_INFINITY, f256::ONE);
        assert!((f256::INFINITY % f256::INFINITY).is_nan());
        assert!((f256::INFINITY % f256::NEG_INFINITY).is_nan());
        assert!((f256::NEG_INFINITY % f256::INFINITY).is_nan());
        assert!((f256::NEG_INFINITY % f256::NEG_INFINITY).is_nan());
    }

    #[test]
    fn test_zero() {
        assert!((f256::ONE % f256::ZERO).is_nan());
        assert_eq!(f256::ZERO % f256::ONE, f256::ZERO);
        assert!((f256::ONE % f256::NEG_ZERO).is_nan());
        assert_eq!(f256::NEG_ZERO % f256::ONE, f256::NEG_ZERO);
        assert!((f256::NEG_ONE % f256::ZERO).is_nan());
        assert_eq!(f256::ZERO % f256::NEG_ONE, f256::ZERO);
        assert!((f256::NEG_ONE % f256::NEG_ZERO).is_nan());
        assert_eq!(f256::NEG_ZERO % f256::NEG_ONE, f256::ZERO);
        assert!((f256::ZERO % f256::ZERO).is_nan());
        assert!((f256::ZERO % f256::NEG_ZERO).is_nan());
        assert!((f256::NEG_ZERO % f256::ZERO).is_nan());
        assert!((f256::NEG_ZERO % f256::NEG_ZERO).is_nan());
    }

    #[test]
    fn test_normal() {
        assert_eq!(f256::ONE % f256::ONE, f256::ZERO);
        assert_eq!(f256::ONE % f256::NEG_ONE, f256::ZERO);
        assert_eq!(f256::NEG_ONE % f256::ONE, f256::ZERO);
        assert_eq!(f256::from(4.0) % f256::TWO, f256::ZERO);
        assert_eq!(f256::from(-5.0) % f256::TWO, f256::NEG_ONE);
        assert_eq!(f256::from(9.5) % f256::from(3.0), f256::from(0.5));
        assert_eq!(f256::from(-9.5) % f256::from(3.0), f256::from(-0.5));
        assert_eq!(f256::from(9.5) % f256::from(-3.0), f256::from(0.5));
        assert_eq!(f256::from(-9.5) % f256::from(-3.0), f256::from(-0.5));
        assert_eq!(f256::from(9.625) % f256::from(2.5), f256::from(2.125));
        assert_eq!(f256::from(-9.625) % f256::from(2.5), f256::from(-2.125));
    }

    #[test]
    fn test_subnormal_1() {
        let x = f256::MIN_GT_ZERO;
        assert_eq!(x % x, f256::ZERO);
        assert_eq!(-x % x, f256::ZERO);
        assert_eq!(x % -x, f256::ZERO);
        assert_eq!(-x % -x, f256::ZERO);
        let y = f256::TWO;
        assert_eq!(x % y, x);
        assert_eq!(-x % y, -x);
        assert_eq!(x % -y, x);
        assert_eq!(-x % -y, -x);
        let z = x * f256::from_str("17.5e78906").unwrap();
        assert!(z.is_finite());
        assert_eq!(z % x, f256::ZERO);
        let z = x * f256::from_str("17.e78902").unwrap();
        assert_eq!(z % x, f256::ZERO);
    }

    #[test]
    fn test_subnormal_2() {
        assert_eq!(f256::MAX % f256::MIN_GT_ZERO, f256::ZERO);
        assert_eq!(f256::MIN % f256::MIN_GT_ZERO, f256::ZERO);
    }

    #[test]
    fn test_subnormal_result_1() {
        let x = f256::from_sign_exp_signif(
            0,
            -144138,
            (
                232007236382389145870169860870637,
                21850787627723933336232264004974193693,
            ),
        );
        assert!(x.is_normal());
        let y = f256::from_sign_exp_signif(
            1,
            -262246,
            (0, 25243622875662406501630802010605),
        );
        assert!(y.is_normal());
        let z = f256::from_sign_exp_signif(
            0,
            -262378,
            (35960771309385850936680005571600, 0),
        );
        assert_eq!(x % y, z);
    }

    #[test]
    fn test_subnormal_result_2() {
        let x = f256::from_sign_exp_signif(
            0,
            -33532,
            (
                622993512440059726197628088986229,
                329531446582523212361253304678338317325,
            ),
        );
        assert!(x.is_normal());
        let y = f256::from_sign_exp_signif(
            1,
            -262246,
            (0, 1551988445904992603287983554167),
        );
        assert!(y.is_subnormal());
        let z = f256::from_sign_exp_signif(
            0,
            -262245,
            (0, 344693067921799021932236086659),
        );
        assert_eq!(x % y, z);
    }

    #[test]
    fn test_normal_2() {
        let x = f256::from_sign_exp_signif(
            1,
            -20167,
            (
                396091524468374439553466833932038,
                287120322474436508255079181112596091133,
            ),
        );
        assert!(x.is_normal());
        let y = f256::from_sign_exp_signif(
            1,
            -125597,
            (
                377276285872660024599382831948763,
                261829273180548883101888490383232063939,
            ),
        );
        assert!(y.is_normal());
        let z = f256::from_sign_exp_signif(
            1,
            -125597,
            (
                225305959063488205115784824377985,
                79960502994656264193093982457981746761,
            ),
        );
        assert_eq!(x % y, z);
    }

    #[test]
    fn test_normal_3() {
        let x = 3.297338e302;
        let y = 1.008297e-297;
        let z = x % y;
        let xx = f256::from(x);
        let yy = f256::from(y);
        let zz = f256::from(z);
        assert_eq!(xx % yy, zz);
    }
}
