// ---------------------------------------------------------------------------
// Copyright:   (c) 2022 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

#[cfg(test)]
mod tests {
    use f256::f256;

    #[test]
    fn test_nan() {
        assert!(f256::NAN.is_nan());
        assert!((-f256::NAN).is_nan());
        assert!(!f256::INFINITY.is_nan());
        assert!(!f256::NEG_INFINITY.is_nan());
        assert!(!f256::ZERO.is_nan());
        assert!(!f256::NEG_ZERO.is_nan());
    }

    #[test]
    fn test_inf() {
        assert!(f256::INFINITY.is_infinite());
        assert!(f256::NEG_INFINITY.is_infinite());
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
    }

    #[test]
    fn test_zero() {
        assert!(f256::ZERO.is_zero());
        assert!(f256::NEG_ZERO.is_zero());
        assert!(!f256::ONE.is_zero());
        assert!(!f256::NEG_ONE.is_zero());
        assert!(!f256::NAN.is_zero());
        assert!(!(-f256::NAN).is_zero());
        assert!(!f256::INFINITY.is_zero());
        assert!(!f256::NEG_INFINITY.is_zero());
    }
}
