// ---------------------------------------------------------------------------
// Copyright:   (c) 2022 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

#[cfg(test)]
mod hypot_tests {
    use core::cmp::Ordering;
    use f256::f256;

    #[test]
    fn test_nan() {
        assert!(f256::NAN.hypot(f256::TWO).is_nan());
        assert!(f256::ONE.hypot(f256::NAN).is_nan());
        assert!(f256::NAN.hypot(f256::NEG_INFINITY).is_nan());
        assert!(f256::INFINITY.hypot(f256::NAN).is_nan());
        assert!(f256::NEG_INFINITY.hypot(f256::NAN).is_nan());
    }

    #[test]
    fn test_inf() {
        assert_eq!(f256::INFINITY.hypot(f256::ONE), f256::INFINITY);
        assert_eq!(f256::TWO.hypot(f256::INFINITY), f256::INFINITY);
        assert_eq!(f256::NEG_INFINITY.hypot(f256::TEN), f256::INFINITY);
        assert_eq!(
            f256::NEG_INFINITY.hypot(f256::NEG_INFINITY),
            f256::INFINITY
        );
        assert_eq!(f256::TEN.hypot(f256::NEG_INFINITY), f256::INFINITY);
    }

    #[test]
    fn test_zero() {
        // Because the normal cmp treats 0 == -0, we have to use total_cmp.
        assert_eq!(
            f256::ZERO.hypot(f256::ZERO).total_cmp(&f256::ZERO),
            Ordering::Equal
        );
        assert_eq!(
            f256::ZERO.hypot(f256::NEG_ZERO).total_cmp(&f256::ZERO),
            Ordering::Equal
        );
        assert_eq!(
            f256::NEG_ZERO.hypot(f256::ZERO).total_cmp(&f256::ZERO),
            Ordering::Equal
        );
        assert_eq!(f256::TWO.hypot(f256::ZERO), f256::TWO);
        assert_eq!(f256::ZERO.hypot(f256::TWO), f256::TWO);
        assert_eq!(f256::NEG_ZERO.hypot(-f256::MAX), f256::MAX);
        assert_eq!(
            f256::MIN_GT_ZERO.hypot(f256::NEG_ZERO),
            f256::MIN_GT_ZERO
        );
    }

    #[test]
    fn test_normal_no_diff_to_non_fused() {
        assert_eq!(f256::ONE.hypot(f256::ONE), f256::TWO.sqrt());
        assert_eq!(f256::ONE.hypot(f256::NEG_ONE), f256::TWO.sqrt());
        assert_eq!(f256::NEG_ONE.hypot(f256::ONE), f256::TWO.sqrt());
        assert_eq!(f256::TWO.hypot(-f256::TEN), f256::from(104.0).sqrt());
    }

    #[test]
    fn test_normal_near_one() {
        let d = f256::EPSILON;
        let x = f256::ONE - d;
        let y = f256::ONE + d;
        let z = x.sum_of_squares(y).sqrt();
        assert_eq!(x.hypot(y), z);
    }

    #[test]
    fn test_normal_near_epsilon() {
        let d = f256::EPSILON.square();
        let x = f256::EPSILON + d;
        let y = f256::EPSILON;
        let z = x.sum_of_squares(y).sqrt();
        assert_eq!(x.hypot(y), z);
    }

    #[test]
    fn test_overflow() {
        let f = f256::MAX;
        let g = f.ulp().mul_pow2(f256::SIGNIFICANT_DIGITS / 2 + 1);
        let z = f.hypot(g);
        assert_eq!(z, f256::INFINITY);
    }

    #[test]
    fn test_no_overflow() {
        let f = f256::MAX;
        let z = f.hypot(f256::ONE);
        assert_eq!(z, f256::MAX);
        let f = f256::MAX / f256::TWO.sqrt();
        let z = f.hypot(f);
        assert_eq!(z, f256::MAX);
    }

    #[test]
    fn test_no_underflow() {
        let f = f256::from_sign_exp_signif(
            1,
            -262378,
            ((1_u128 << 108) - 1, u128::MAX),
        );
        assert_eq!((-f).hypot(f), f.abs() * f256::TWO.sqrt());
        let f = f256::MIN_GT_ZERO;
        // ⌈f⋅√2⌋₂₃₆ = f
        assert_eq!((-f).hypot(f), f);
    }
}
