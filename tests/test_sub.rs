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
    fn test_subnormal() {
        assert_eq!(f256::MAX - f256::MIN_GT_ZERO, f256::MAX);
        assert_eq!(f256::MIN - f256::MIN_GT_ZERO, f256::MIN);
        assert_eq!(f256::MIN_GT_ZERO - f256::MAX, f256::MIN);
        assert_eq!(f256::MIN_GT_ZERO - f256::MIN, f256::MAX);
        assert_eq!(f256::ONE - f256::MIN_GT_ZERO, f256::ONE);
        assert_eq!(f256::MIN_GT_ZERO - f256::ONE, f256::NEG_ONE);
        assert_eq!(f256::MIN_GT_ZERO - f256::MIN_GT_ZERO, f256::ZERO);
        // TODO: sub two subnormals giving subnormal result
    }

    #[test]
    fn test_overflow() {
        assert_eq!(f256::MIN - f256::MAX, f256::NEG_INFINITY);
        assert_eq!(f256::MIN - f256::ONE, f256::MIN);
    }
}
