// ---------------------------------------------------------------------------
// Copyright:   (c) 2022 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

#[cfg(test)]
mod abs_tests {
    use f256::f256;

    #[test]
    fn test_nan() {
        assert!(f256::NAN.abs().is_nan());
        assert!((-f256::NAN).abs().is_nan());
    }

    #[test]
    fn test_inf() {
        assert_eq!(f256::INFINITY.abs(), f256::INFINITY);
        assert_eq!(f256::NEG_INFINITY.abs(), f256::INFINITY);
    }

    #[test]
    fn test_zero() {
        assert_eq!(f256::ZERO.abs(), f256::ZERO);
        assert_eq!(f256::NEG_ZERO.abs(), f256::ZERO);
    }

    #[test]
    fn test_normal() {
        let f = f256::from(17.625_f64);
        let g = -f;
        assert_eq!(f.abs(), f);
        assert_eq!(g.abs(), f);
    }

    #[test]
    fn test_subnormal() {
        let f = f256::MIN_GT_ZERO;
        assert_eq!(f.abs(), f);
        assert_eq!((-f).abs(), f);
    }
}

#[cfg(test)]
mod trunc_tests {
    use f256::f256;

    #[test]
    fn test_nan() {
        assert!(f256::NAN.trunc().is_nan());
        assert!((-f256::NAN).trunc().is_nan());
    }

    #[test]
    fn test_inf() {
        assert_eq!(f256::INFINITY.trunc(), f256::INFINITY);
        assert_eq!(f256::NEG_INFINITY.trunc(), f256::NEG_INFINITY);
    }

    #[test]
    fn test_zero() {
        assert_eq!(f256::ZERO.trunc(), f256::ZERO);
        assert_eq!(f256::NEG_ZERO.trunc(), f256::NEG_ZERO);
    }

    #[test]
    fn test_normal() {
        let f = f256::from(17);
        let g = f256::from(17.625_f64);
        let h = -g;
        assert_eq!(f.trunc(), f);
        assert_eq!(g.trunc(), f);
        assert_eq!(h.trunc(), -f);
    }

    #[test]
    fn test_lt_0() {
        let f = f256::from(0.99999_f64);
        assert_eq!(f.trunc(), f256::ZERO);
        let e = f256::EPSILON;
        assert_eq!(e.trunc(), f256::ZERO);
    }

    #[test]
    fn test_gt_2_pow_237() {
        let f = f256::from(1.3097428e71_f64);
        assert_eq!(f.trunc(), f);
    }

    #[test]
    fn test_subnormal() {
        let f = f256::MIN_GT_ZERO;
        assert_eq!(f.trunc(), f256::ZERO);
    }
}

#[cfg(test)]
mod fract_tests {
    use f256::f256;

    #[test]
    fn test_nan() {
        assert!(f256::NAN.fract().is_nan());
        assert!((-f256::NAN).fract().is_nan());
    }

    #[test]
    fn test_inf() {
        assert!(f256::INFINITY.fract().is_nan());
        assert!(f256::NEG_INFINITY.fract().is_nan());
    }

    #[test]
    fn test_zero() {
        assert_eq!(f256::ZERO.fract(), f256::ZERO);
        assert_eq!(f256::NEG_ZERO.fract(), f256::ZERO);
    }

    #[test]
    fn test_normal() {
        let f = f256::from(17);
        let g = f256::from(17.625_f64);
        let h = -g;
        assert_eq!(f.fract(), f256::ZERO);
        assert_eq!(g.fract(), g - f);
        assert_eq!(h.fract(), f - g);
    }

    #[test]
    fn test_lt_0() {
        let f = f256::from(0.99999_f64);
        assert_eq!(f.fract(), f);
        let e = f256::EPSILON;
        assert_eq!(e.fract(), f256::EPSILON);
    }

    #[test]
    fn test_subnormal() {
        let f = f256::MIN_GT_ZERO;
        assert_eq!(f.fract(), f256::MIN_GT_ZERO);
    }
}
