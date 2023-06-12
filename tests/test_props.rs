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
mod property_tests {
    use core::num::FpCategory;

    use f256::f256;

    #[test]
    fn test_nan() {
        assert!(f256::NAN.is_nan());
        assert!((-f256::NAN).is_nan());
        assert_eq!(f256::NAN.classify(), FpCategory::Nan);
        assert!(!f256::INFINITY.is_nan());
        assert!(!f256::NEG_INFINITY.is_nan());
        assert!(!f256::ZERO.is_nan());
        assert!(!f256::NEG_ZERO.is_nan());
        assert!(!f256::TWO.is_nan());
        assert!(!f256::from(-27).is_nan());
    }

    #[test]
    fn test_inf() {
        assert!(f256::INFINITY.is_infinite());
        assert_eq!(f256::INFINITY.classify(), FpCategory::Infinite);
        assert!(f256::NEG_INFINITY.is_infinite());
        assert_eq!(f256::NEG_INFINITY.classify(), FpCategory::Infinite);
        assert!(!f256::NAN.is_infinite());
        assert!(!f256::NEG_ZERO.is_infinite());
        assert!(!f256::ZERO.is_infinite());
        assert!(!f256::ONE.is_infinite());
        assert!(!f256::INFINITY.is_finite());
        assert!(!f256::NEG_INFINITY.is_finite());
        assert!(!f256::NAN.is_finite());
        assert!(f256::NEG_ZERO.is_finite());
        assert!(f256::ZERO.is_finite());
        assert!(f256::ONE.is_finite());
        assert!(f256::from(-380).is_finite());
        assert!(!f256::from(-380).is_infinite());
    }

    #[test]
    fn test_zero() {
        assert!(f256::ZERO.eq_zero());
        assert_eq!(f256::ZERO.classify(), FpCategory::Zero);
        assert!(f256::NEG_ZERO.eq_zero());
        assert_eq!(f256::NEG_ZERO.classify(), FpCategory::Zero);
        assert!(!f256::ONE.eq_zero());
        assert!(!f256::NEG_ONE.eq_zero());
        assert!(!f256::NAN.eq_zero());
        assert!(!(-f256::NAN).eq_zero());
        assert!(!f256::INFINITY.eq_zero());
        assert!(!f256::NEG_INFINITY.eq_zero());
        assert!(f256::from(0.0_f32).eq_zero());
        assert!(f256::from(-0.0_f64).eq_zero());
        assert!(!f256::from(0.001_f64).eq_zero());
        assert!(!f256::MIN_GT_ZERO.eq_zero());
        assert!(!(-f256::MIN_GT_ZERO).eq_zero());
    }

    #[test]
    fn test_normal() {
        assert!(!f256::ZERO.is_normal());
        assert!(!f256::NEG_ZERO.is_normal());
        assert!(f256::ONE.is_normal());
        assert_eq!(f256::ONE.classify(), FpCategory::Normal);
        assert!(f256::NEG_ONE.is_normal());
        assert_eq!(f256::NEG_ONE.classify(), FpCategory::Normal);
        assert!(f256::EPSILON.is_normal());
        assert_eq!(f256::EPSILON.classify(), FpCategory::Normal);
        assert!(f256::MAX.is_normal());
        assert!(f256::MIN.is_normal());
        assert!(f256::MIN_POSITIVE.is_normal());
        assert!(!f256::NAN.is_normal());
        assert!(!(-f256::NAN).is_normal());
        assert!(!f256::INFINITY.is_normal());
        assert!(!f256::NEG_INFINITY.is_normal());
        assert!(f256::from(f64::MAX).is_normal());
        assert!(f256::from(1e-312_f64).is_normal());
        assert!(!f256::MIN_GT_ZERO.is_normal());
        assert!(!(-f256::MIN_GT_ZERO).is_normal());
    }

    #[test]
    fn test_subnormal() {
        assert!(!f256::ZERO.is_subnormal());
        assert!(!f256::NEG_ZERO.is_subnormal());
        assert!(!f256::ONE.is_subnormal());
        assert!(!f256::NEG_ONE.is_subnormal());
        assert!(!f256::EPSILON.is_subnormal());
        assert!(!f256::MAX.is_subnormal());
        assert!(!f256::MIN.is_subnormal());
        assert!(!f256::MIN_POSITIVE.is_subnormal());
        assert!(!f256::NAN.is_subnormal());
        assert!(!(-f256::NAN).is_subnormal());
        assert!(!f256::INFINITY.is_subnormal());
        assert!(!f256::NEG_INFINITY.is_subnormal());
        assert!(!f256::from(f64::MAX).is_subnormal());
        assert!(!f256::from(1e-312_f64).is_subnormal());
        assert!(f256::MIN_GT_ZERO.is_subnormal());
        assert_eq!(f256::MIN_GT_ZERO.classify(), FpCategory::Subnormal);
        assert!((-f256::MIN_GT_ZERO).is_subnormal());
    }

    #[test]
    fn test_special() {
        assert!(f256::ZERO.is_special());
        assert!(f256::NEG_ZERO.is_special());
        assert!(!f256::ONE.is_special());
        assert!(!f256::NEG_ONE.is_special());
        assert!(!f256::EPSILON.is_special());
        assert!(!f256::MAX.is_special());
        assert!(!f256::MIN.is_special());
        assert!(!f256::MIN_POSITIVE.is_special());
        assert!(f256::NAN.is_special());
        assert!((-f256::NAN).is_special());
        assert!(f256::INFINITY.is_special());
        assert!(f256::NEG_INFINITY.is_special());
        assert!(!f256::from(f64::MAX).is_special());
        assert!(!f256::from(1e-312_f64).is_special());
        assert!(!f256::MIN_GT_ZERO.is_special());
        assert!(!(-f256::MIN_GT_ZERO).is_special());
    }
}
