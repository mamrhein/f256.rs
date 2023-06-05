// ---------------------------------------------------------------------------
// Copyright:   (c) 2022 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

#[cfg(test)]
mod div_tests {
    use f256::f256;

    #[test]
    fn test_nan() {
        assert!((f256::NAN / f256::ONE).is_nan());
        assert!((f256::ONE / f256::NAN).is_nan());
        assert!((f256::NAN / f256::NAN).is_nan());
        assert!((f256::NAN / f256::INFINITY).is_nan());
        assert!((f256::INFINITY / f256::NAN).is_nan());
        assert!((f256::NAN / f256::NEG_INFINITY).is_nan());
        assert!((f256::NEG_INFINITY / f256::NAN).is_nan());
    }

    #[test]
    fn test_inf() {
        assert_eq!(f256::INFINITY / f256::ONE, f256::INFINITY);
        assert_eq!(f256::ONE / f256::INFINITY, f256::ZERO);
        assert_eq!(f256::NEG_INFINITY / f256::ONE, f256::NEG_INFINITY);
        assert_eq!(f256::ONE / f256::NEG_INFINITY, f256::NEG_ZERO);
        assert!((f256::INFINITY / f256::INFINITY).is_nan());
        assert!((f256::INFINITY / f256::NEG_INFINITY).is_nan());
        assert!((f256::NEG_INFINITY / f256::INFINITY).is_nan());
        assert!((f256::NEG_INFINITY / f256::NEG_INFINITY).is_nan());
    }

    #[test]
    fn test_zero() {
        assert_eq!(f256::ONE / f256::ZERO, f256::INFINITY);
        assert_eq!(f256::ZERO / f256::ONE, f256::ZERO);
        assert_eq!(f256::ONE / f256::NEG_ZERO, f256::NEG_INFINITY);
        assert_eq!(f256::NEG_ZERO / f256::ONE, f256::NEG_ZERO);
        assert_eq!(f256::NEG_ONE / f256::ZERO, f256::NEG_INFINITY);
        assert_eq!(f256::ZERO / f256::NEG_ONE, f256::NEG_ZERO);
        assert_eq!(f256::NEG_ONE / f256::NEG_ZERO, f256::INFINITY);
        assert_eq!(f256::NEG_ZERO / f256::NEG_ONE, f256::ZERO);
        assert!((f256::ZERO / f256::ZERO).is_nan());
        assert!((f256::ZERO / f256::NEG_ZERO).is_nan());
        assert!((f256::NEG_ZERO / f256::ZERO).is_nan());
        assert!((f256::NEG_ZERO / f256::NEG_ZERO).is_nan());
    }

    #[test]
    fn test_normal_1() {
        assert_eq!(f256::ONE / f256::ONE, f256::ONE);
        assert_eq!(f256::ONE / f256::NEG_ONE, f256::NEG_ONE);
        assert_eq!(f256::NEG_ONE / f256::ONE, f256::NEG_ONE);
        assert_eq!(f256::from(4.0) / f256::TWO, f256::TWO);
        assert_eq!(f256::from(9.625) / f256::from(2.75), f256::from(3.5));
    }

    #[test]
    fn test_normal_2() {
        let x = f256::from_sign_exp_signif(
            0,
            99038,
            (
                530271203563337380805884060470794,
                7992319161702241479140416673459104095,
            ),
        );
        let y = f256::from_sign_exp_signif(
            0,
            6048,
            (
                544583933317535774300137218755011,
                316149147203647116175450086187958872287,
            ),
        );
        let z = f256::from_sign_exp_signif(
            0,
            92755,
            (
                157994786018365171004830363945030,
                126953907264130766956444977049689243903,
            ),
        );
        assert_eq!(x / y, z)
    }

    #[test]
    fn test_normal_3() {
        let x = f256::from_sign_exp_signif(
            1,
            -260476,
            (
                223521899081924826801773646089388,
                300016796735507732683379365788054907187,
            ),
        );
        let y = f256::from_sign_exp_signif(
            0,
            -231731,
            (
                45137657171945564220560630934455,
                244494144206352980600118687098334874849,
            ),
        );
        let z = f256::from_sign_exp_signif(
            1,
            -28978,
            (
                200877183115451531153822352949160,
                279577911722371923290142768737999722677,
            ),
        );
        assert_eq!(x / y, z)
    }

    #[test]
    fn test_normal_4() {
        let x = f256::from_sign_exp_signif(
            1,
            194536,
            (
                529222974574459971744886741366716,
                105035947865812708090400053927181373375,
            ),
        );
        let y = f256::from_sign_exp_signif(
            1,
            16118,
            (
                435121311940907893407190643926091,
                249061112421671143573157368046380011417,
            ),
        );
        let z = f256::from_sign_exp_signif(
            0,
            178182,
            (
                394700672108282726135590318411562,
                24287937956974581624147545956830544655,
            ),
        );
        assert_eq!(x / y, z)
    }

    #[test]
    fn test_subnormal() {
        let x = f256::MIN_GT_ZERO;
        assert_eq!(x / x, f256::ONE);
        assert_eq!(-x / x, f256::NEG_ONE);
        assert_eq!(x / -x, f256::NEG_ONE);
        assert_eq!(-x / -x, f256::ONE);
        let y = f256::TWO;
        assert_eq!(x / y, f256::ZERO);
        assert_eq!(-x / y, f256::NEG_ZERO);
        assert_eq!(x / -y, f256::NEG_ZERO);
        assert_eq!(-x / -y, f256::ZERO);
        let y = f256::from(0.5);
        let z = x + x;
        assert_eq!(x / y, z);
    }

    #[test]
    fn test_overflow_1() {
        assert_eq!(f256::MAX / f256::MIN_GT_ZERO, f256::INFINITY);
        assert_eq!(f256::MIN / f256::MIN_GT_ZERO, f256::NEG_INFINITY);
    }

    #[test]
    fn test_overflow_2() {
        let x = f256::from_sign_exp_signif(
            1,
            247449,
            (
                222581619208314555639692392399675,
                294685291944275559831296488670417115667,
            ),
        );
        let y = f256::from_sign_exp_signif(
            0,
            -14694,
            (
                85152708844161637247465184931530,
                24205953785655276835031997231039835903,
            ),
        );
        let z = f256::from_sign_exp_signif(1, 262144, (0, 0));
        assert_eq!(x / y, z)
    }
}
