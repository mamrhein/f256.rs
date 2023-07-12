// ---------------------------------------------------------------------------
// Copyright:   (c) 2022 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

#[cfg(test)]
mod sum_of_squares_tests {
    use core::cmp::Ordering;

    use f256::f256;

    #[test]
    fn test_nan() {
        assert!(f256::NAN.sum_of_squares(f256::TWO).is_nan());
        assert!(f256::ONE.sum_of_squares(f256::NAN).is_nan());
        assert!(f256::NAN.sum_of_squares(f256::NEG_INFINITY).is_nan());
        assert!(f256::INFINITY.sum_of_squares(f256::NAN).is_nan());
        assert!(f256::NEG_INFINITY.sum_of_squares(f256::NAN).is_nan());
    }

    #[test]
    fn test_inf() {
        assert_eq!(f256::INFINITY.sum_of_squares(f256::ONE), f256::INFINITY);
        assert_eq!(f256::TWO.sum_of_squares(f256::INFINITY), f256::INFINITY);
        assert_eq!(
            f256::NEG_INFINITY.sum_of_squares(f256::TEN),
            f256::INFINITY
        );
        assert_eq!(
            f256::NEG_INFINITY.sum_of_squares(f256::NEG_INFINITY),
            f256::INFINITY
        );
        assert_eq!(
            f256::TEN.sum_of_squares(f256::NEG_INFINITY),
            f256::INFINITY
        );
    }

    #[test]
    fn test_zero() {
        // Because the normal cmp treats 0 == -0, we have to use total_cmp.
        assert_eq!(
            f256::ZERO.sum_of_squares(f256::ZERO).total_cmp(&f256::ZERO),
            Ordering::Equal
        );
        assert_eq!(
            f256::ZERO
                .sum_of_squares(f256::NEG_ZERO)
                .total_cmp(&f256::ZERO),
            Ordering::Equal
        );
        assert_eq!(
            f256::NEG_ZERO
                .sum_of_squares(f256::ZERO)
                .total_cmp(&f256::ZERO),
            Ordering::Equal
        );
        assert_eq!(f256::ONE.sum_of_squares(f256::ZERO), f256::ONE);
        assert_eq!(f256::ZERO.sum_of_squares(f256::ONE), f256::ONE);
        assert_eq!(f256::NEG_ZERO.sum_of_squares(f256::ONE), f256::ONE);
    }

    #[test]
    fn test_normal_no_diff_to_non_fused() {
        assert_eq!(f256::ONE.sum_of_squares(f256::ONE), f256::TWO);
        assert_eq!(f256::ONE.sum_of_squares(f256::NEG_ONE), f256::TWO);
        assert_eq!(f256::NEG_ONE.sum_of_squares(f256::ONE), f256::TWO);
        assert_eq!(f256::TWO.sum_of_squares(-f256::TEN), f256::from(104.0));
    }

    #[test]
    fn test_normal_near_one() {
        let d = f256::EPSILON;
        let x = f256::ONE - d;
        let y = f256::ONE + d;
        let z = d.square_add(f256::TWO);
        assert_eq!(x.sum_of_squares(y), z);
    }

    #[test]
    fn test_normal_near_epsilon() {
        let d = f256::EPSILON.square();
        let x = f256::EPSILON + d;
        let y = f256::EPSILON;
        let z = f256::TWO * d + f256::TWO * d * y + d.square();
        assert_eq!(x.sum_of_squares(y), z);
    }

    #[test]
    fn test_square_overflow() {
        let f = f256::MAX;
        let z = f.sum_of_squares(f256::ONE);
        assert_eq!(z, f256::INFINITY);
        let z = f256::ZERO.sum_of_squares(-f);
        assert_eq!(z, f256::INFINITY);
    }

    #[test]
    fn test_square_underflow() {
        let f = f256::MIN_GT_ZERO;
        assert_eq!(-f.sum_of_squares(f256::ZERO), f256::ZERO);
        assert_eq!(-f.sum_of_squares(f), f256::ZERO);
        let z = f.sum_of_squares(f256::ONE);
        assert_eq!(z, f256::ONE);
    }
}
