// ---------------------------------------------------------------------------
// Copyright:   (c) 2022 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

#[cfg(test)]
mod add_tests {
    use core::cmp::Ordering;
    use std::ops::Div;

    use f256::f256;

    #[test]
    fn test_nan() {
        assert!(f256::NAN.mul_add(f256::TWO, f256::ONE).is_nan());
        assert!(f256::ONE.mul_add(f256::NAN, f256::TWO).is_nan());
        assert!(f256::ONE.mul_add(f256::TWO, f256::NAN).is_nan());
        assert!(f256::NAN.mul_add(f256::NAN, f256::ONE).is_nan());
        assert!(f256::NAN.mul_add(f256::TWO, f256::INFINITY).is_nan());
        assert!(f256::INFINITY.mul_add(f256::TWO, f256::NAN).is_nan());
        assert!(f256::NAN.mul_add(f256::TWO, f256::NEG_INFINITY).is_nan());
        assert!(f256::NEG_INFINITY.mul_add(f256::TWO, f256::NAN).is_nan());
        assert!(f256::NEG_INFINITY.mul_add(f256::NAN, f256::TWO).is_nan());
    }

    #[test]
    fn test_inf() {
        assert_eq!(
            f256::INFINITY.mul_add(f256::ONE, f256::INFINITY),
            f256::INFINITY
        );
        assert_eq!(
            f256::INFINITY.mul_add(f256::TWO, f256::ONE),
            f256::INFINITY
        );
        assert_eq!(
            f256::TWO.mul_add(f256::TWO, f256::INFINITY),
            f256::INFINITY
        );
        assert_eq!(
            f256::NEG_INFINITY.mul_add(f256::TEN, f256::NEG_INFINITY),
            f256::NEG_INFINITY
        );
        assert_eq!(
            f256::NEG_INFINITY.mul_add(f256::ONE, f256::TEN),
            f256::NEG_INFINITY
        );
        assert_eq!(
            f256::TEN.mul_add(f256::ONE, f256::NEG_INFINITY),
            f256::NEG_INFINITY
        );
        assert_eq!(
            f256::NEG_INFINITY.mul_add(f256::NEG_ONE, f256::INFINITY),
            f256::INFINITY
        );
        assert!(f256::INFINITY
            .mul_add(f256::INFINITY, f256::NEG_INFINITY)
            .is_nan());
        assert!(f256::NEG_INFINITY
            .mul_add(f256::ONE, f256::INFINITY)
            .is_nan());
        assert!(f256::INFINITY
            .mul_add(f256::NEG_INFINITY, f256::INFINITY)
            .is_nan());
    }

    #[test]
    fn test_zero() {
        // Because the normal cmp treats 0 == -0, we have to use total_cmp.
        assert_eq!(
            f256::ZERO
                .mul_add(f256::ZERO, f256::ZERO)
                .total_cmp(&f256::ZERO),
            Ordering::Equal
        );
        assert_eq!(
            f256::ZERO
                .mul_add(f256::ZERO, f256::NEG_ZERO)
                .total_cmp(&f256::ZERO),
            Ordering::Equal
        );
        assert_eq!(
            f256::NEG_ZERO
                .mul_add(f256::ZERO, f256::ZERO)
                .total_cmp(&f256::ZERO),
            Ordering::Equal
        );
        assert_eq!(
            f256::NEG_ZERO
                .mul_add(f256::ZERO, f256::NEG_ZERO)
                .total_cmp(&f256::NEG_ZERO),
            Ordering::Equal
        );
        assert_eq!(
            f256::ZERO
                .mul_add(f256::NEG_ZERO, f256::NEG_ZERO)
                .total_cmp(&f256::NEG_ZERO),
            Ordering::Equal
        );
        assert_eq!(
            f256::ZERO
                .mul_add(f256::ONE, f256::ZERO)
                .total_cmp(&f256::ZERO),
            Ordering::Equal
        );
        assert_eq!(
            f256::ZERO
                .mul_add(f256::ONE, f256::NEG_ZERO)
                .total_cmp(&f256::ZERO),
            Ordering::Equal
        );
        assert_eq!(
            f256::NEG_ZERO
                .mul_add(f256::ONE, f256::ZERO)
                .total_cmp(&f256::ZERO),
            Ordering::Equal
        );
        assert_eq!(
            f256::NEG_ZERO
                .mul_add(f256::ONE, f256::NEG_ZERO)
                .total_cmp(&f256::NEG_ZERO),
            Ordering::Equal
        );
        assert_eq!(f256::ONE.mul_add(f256::ONE, f256::ZERO), f256::ONE);
        assert_eq!(f256::ZERO.mul_add(f256::ONE, f256::ONE), f256::ONE);
        assert_eq!(f256::ONE.mul_add(f256::ONE, f256::NEG_ZERO), f256::ONE);
        assert_eq!(f256::NEG_ZERO.mul_add(f256::ONE, f256::ONE), f256::ONE);
    }

    #[test]
    fn test_normal_no_diff_to_non_fused() {
        assert_eq!(f256::ONE.mul_add(f256::ONE, f256::ONE), f256::TWO);
        assert_eq!(f256::ONE.mul_add(f256::ONE, f256::NEG_ONE), f256::ZERO);
        assert_eq!(f256::NEG_ONE.mul_add(f256::ONE, f256::ONE), f256::ZERO);
        assert_eq!(f256::TWO.mul_add(f256::TWO, f256::ONE), f256::from(5.0));
        assert_eq!(
            f256::from(3.5).mul_add(f256::ONE, f256::from(3.5)),
            f256::from(7.0)
        );
        assert_eq!(
            f256::from(3.5).mul_add(f256::TWO, f256::from(-3.5)),
            f256::from(3.5)
        );
        assert_eq!(
            f256::from(-3.5).mul_add(f256::from(3.0), f256::from(3.5)),
            f256::from(-7.0)
        );
        assert_eq!(f256::MAX.mul_add(f256::ONE, f256::MIN), f256::ZERO);
        assert_eq!(f256::MIN.mul_add(f256::ONE, f256::MAX), f256::ZERO);
        assert_eq!(f256::MAX.mul_add(f256::ONE, f256::EPSILON), f256::MAX);
        assert_eq!(f256::MIN.mul_add(f256::ONE, f256::EPSILON), f256::MIN);
        assert_eq!(
            f256::MAX
                .div(f256::TWO)
                .mul_add(f256::TWO, f256::MIN_GT_ZERO),
            f256::MAX
        );
        assert_eq!(
            f256::MIN
                .div(f256::TEN)
                .mul_add(f256::TEN, f256::MIN_GT_ZERO),
            f256::MIN
        );
    }

    #[test]
    fn test_addend_zero() {
        let x = f256::from_sign_exp_signif(0, -7192, (2891_u128, 1_u128));
        let y = f256::from_sign_exp_signif(0, -83, (555_u128, 10001_u128));
        assert_eq!(x.mul_add(y, f256::ZERO), x * y);
    }

    #[test]
    fn test_one_times_one_plus_one() {
        let f = f256::ONE;
        assert_eq!(f.mul_add(f, f), f256::TWO);
    }

    #[test]
    fn test_two_times_two_plus_one() {
        let f = f256::TWO;
        assert_eq!(f.mul_add(f, f256::ONE), f256::from(5));
    }

    #[test]
    fn test_one_and_a_half_times_two_and_a_half_minus_two() {
        let x = f256::from(1.5);
        let y = f256::from(2.5);
        let a = -f256::TWO;
        let z = x * y + a;
        assert_eq!(x.mul_add(y, a), z);
    }

    #[test]
    fn test_normal_near_one() {
        let p = -237;
        let d = f256::from_sign_exp_signif(0, p, (0_u128, 1_u128));
        let x = f256::ONE - d;
        let y = x;
        let a = f256::TWO * d - f256::ONE;
        let z = f256::from_sign_exp_signif(0, 2 * p, (0_u128, 1_u128));
        assert_eq!(x.mul_add(y, a), z);
    }

    #[test]
    fn test_normal_near_epsilon() {
        let d = f256::EPSILON * f256::EPSILON;
        let x = f256::EPSILON + d;
        let y = f256::EPSILON - f256::TWO * d;
        let a = -d + f256::EPSILON * d;
        let z = -f256::TWO * d * d;
        assert_eq!(x.mul_add(y, a), z);
    }

    #[test]
    fn test_addendtoo_small() {
        let x = f256::from_sign_exp_signif(1, 237, (0_u128, 1_u128));
        let y = f256::TWO;
        let a = -f256::EPSILON;
        let z = x * y;
        assert_eq!(x.mul_add(y, a), z);
    }

    #[test]
    fn test_product_anchored() {
        let x = f256::TEN;
        let y = f256::TWO;
        let a = -f256::EPSILON;
        let z = x * y;
        assert_eq!(x.mul_add(y, a), z);
    }

    #[test]
    fn test_addend_anchored() {
        let x = f256::EPSILON * f256::EPSILON;
        let y = f256::TWO;
        let a = f256::from_sign_exp_signif(1, -234, (0_u128, 1_u128));
        let z = a + f256::from_sign_exp_signif(0, -471, (0_u128, 1_u128));
        assert_eq!(x.mul_add(y, a), z);
    }

    #[test]
    fn test_product_too_small() {
        let x = f256::EPSILON * f256::EPSILON;
        let y = f256::TWO;
        let a = f256::from_sign_exp_signif(1, -230, (0_u128, 1_u128));
        let z = a;
        assert_eq!(x.mul_add(y, a), z);
    }

    #[test]
    fn test_prod_overflow() {
        let f = f256::MAX;
        let z = f.mul_add(f, f256::NEG_INFINITY);
        assert_eq!(z, f256::NEG_INFINITY);
        let z = f.mul_add(-f, f256::INFINITY);
        assert_eq!(z, f256::INFINITY);
    }

    #[test]
    fn test_prod_underflow() {
        let f = f256::MIN_GT_ZERO;
        assert_eq!(
            (-f * f + f256::ZERO).total_cmp(&f256::ZERO),
            Ordering::Equal
        );
        let z = f.mul_add(-f, f256::ZERO);
        assert_eq!(z.total_cmp(&f256::NEG_ZERO), Ordering::Equal);
    }
}
