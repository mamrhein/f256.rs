// ---------------------------------------------------------------------------
// Copyright:   (c) 2022 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

extern crate core;

#[cfg(test)]
mod partial_eq_tests {
    use f256::f256;

    #[test]
    fn test_ne_nan() {
        assert_ne!(f256::NAN, f256::NAN);
        assert_ne!(f256::NAN, f256::ONE);
        assert_ne!(f256::ONE, f256::NAN);
        assert_ne!(f256::NAN, f256::INFINITY);
        assert_ne!(f256::INFINITY, f256::NAN);
        assert_ne!(f256::NAN, f256::NEG_INFINITY);
        assert_ne!(f256::NEG_INFINITY, f256::NAN);
    }

    #[test]
    fn test_zeroes() {
        assert_eq!(f256::ZERO, f256::ZERO);
        assert_eq!(f256::ZERO, f256::NEG_ZERO);
        assert_eq!(f256::NEG_ZERO, f256::ZERO);
        assert_eq!(f256::NEG_ZERO, f256::NEG_ZERO);
    }
    #[test]
    fn test_eq() {
        assert_eq!(f256::INFINITY, f256::INFINITY);
        assert_eq!(f256::NEG_INFINITY, f256::NEG_INFINITY);
        assert_eq!(f256::ONE, f256::ONE);
        assert_eq!(f256::TWO, f256::TWO);
        assert_ne!(f256::INFINITY, f256::ONE);
        assert_ne!(f256::ONE, f256::INFINITY);
        assert_ne!(f256::NEG_INFINITY, f256::ONE);
        assert_ne!(f256::ONE, f256::NEG_INFINITY);
        assert_ne!(f256::ONE, f256::TWO);
        assert_ne!(f256::TWO, f256::ONE);
        assert_ne!(f256::INFINITY, f256::NEG_INFINITY);
        assert_ne!(f256::NEG_INFINITY, f256::INFINITY);
    }
}

#[cfg(test)]
mod partial_ord_tests {
    use core::cmp::Ordering;

    use f256::f256;

    #[test]
    fn test_nan() {
        assert!(f256::NAN.partial_cmp(&f256::NAN).is_none());
        assert!(f256::NAN.partial_cmp(&f256::ONE).is_none());
        assert!(f256::ONE.partial_cmp(&f256::NAN).is_none());
        assert!(f256::NAN.partial_cmp(&f256::INFINITY).is_none());
        assert!(f256::INFINITY.partial_cmp(&f256::NAN).is_none());
        assert!(f256::NAN.partial_cmp(&f256::NEG_INFINITY).is_none());
        assert!(f256::NEG_INFINITY.partial_cmp(&f256::NAN).is_none());
    }

    #[test]
    fn test_zeroes() {
        assert_eq!(f256::ZERO.partial_cmp(&f256::ZERO), Some(Ordering::Equal));
        assert_eq!(
            f256::ZERO.partial_cmp(&f256::NEG_ZERO),
            Some(Ordering::Equal)
        );
        assert_eq!(
            f256::NEG_ZERO.partial_cmp(&f256::ZERO),
            Some(Ordering::Equal)
        );
        assert_eq!(
            f256::NEG_ZERO.partial_cmp(&f256::NEG_ZERO),
            Some(Ordering::Equal)
        );
    }
    #[test]
    fn test_ordering() {
        assert!(f256::NEG_INFINITY < f256::INFINITY);
        assert!(f256::ONE <= f256::ONE);
        assert!(f256::TWO > f256::ONE);
        assert!(f256::INFINITY > f256::ONE);
        assert!(f256::ONE < f256::INFINITY);
        assert!(f256::NEG_ZERO < f256::ONE);
        assert!(f256::ONE > f256::NEG_INFINITY);
        assert!(f256::ONE <= f256::TWO);
        assert!(f256::TWO >= f256::ZERO);
        assert!(f256::INFINITY > f256::NEG_INFINITY);
        assert!(f256::NEG_INFINITY < f256::INFINITY);
    }
}
