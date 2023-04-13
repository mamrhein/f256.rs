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
    fn test_normal() {
        assert_eq!(f256::ONE / f256::ONE, f256::ONE);
        assert_eq!(f256::ONE / f256::NEG_ONE, f256::NEG_ONE);
        assert_eq!(f256::NEG_ONE / f256::ONE, f256::NEG_ONE);
        assert_eq!(f256::from(4.0) / f256::TWO, f256::TWO);
        assert_eq!(f256::from(9.625) / f256::from(2.75), f256::from(3.5));
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
