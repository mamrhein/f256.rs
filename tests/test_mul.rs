// ---------------------------------------------------------------------------
// Copyright:   (c) 2022 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

#[cfg(test)]
mod mul_tests {
    use f256::f256;

    #[test]
    fn test_nan() {
        assert!((f256::NAN * f256::ONE).is_nan());
        assert!((f256::ONE * f256::NAN).is_nan());
        assert!((f256::NAN * f256::NAN).is_nan());
        assert!((f256::NAN * f256::INFINITY).is_nan());
        assert!((f256::INFINITY * f256::NAN).is_nan());
        assert!((f256::NAN * f256::NEG_INFINITY).is_nan());
        assert!((f256::NEG_INFINITY * f256::NAN).is_nan());
    }

    #[allow(clippy::cognitive_complexity)]
    #[test]
    fn test_inf() {
        assert_eq!(f256::INFINITY * f256::INFINITY, f256::INFINITY);
        assert_eq!(f256::INFINITY * f256::ONE, f256::INFINITY);
        assert_eq!(f256::ONE * f256::INFINITY, f256::INFINITY);
        assert_eq!(f256::NEG_INFINITY * f256::NEG_INFINITY, f256::INFINITY);
        assert_eq!(f256::NEG_INFINITY * f256::ONE, f256::NEG_INFINITY);
        assert_eq!(f256::ONE * f256::NEG_INFINITY, f256::NEG_INFINITY);
        assert_eq!((f256::INFINITY * f256::NEG_INFINITY), f256::NEG_INFINITY);
        assert_eq!((f256::NEG_INFINITY * f256::INFINITY), f256::NEG_INFINITY);
        assert!((f256::ZERO * f256::INFINITY).is_nan());
        assert!((f256::INFINITY * f256::ZERO).is_nan());
        assert!((f256::ZERO * f256::NEG_INFINITY).is_nan());
        assert!((f256::NEG_INFINITY * f256::ZERO).is_nan());
        assert!((f256::NEG_ZERO * f256::INFINITY).is_nan());
        assert!((f256::INFINITY * f256::NEG_ZERO).is_nan());
        assert!((f256::NEG_ZERO * f256::NEG_INFINITY).is_nan());
        assert!((f256::NEG_INFINITY * f256::NEG_ZERO).is_nan());
    }

    #[test]
    fn test_zero() {
        assert_eq!(f256::ZERO * f256::ZERO, f256::ZERO);
        assert_eq!(f256::ZERO * f256::NEG_ZERO, f256::NEG_ZERO);
        assert_eq!(f256::NEG_ZERO * f256::ZERO, f256::NEG_ZERO);
        assert_eq!(f256::NEG_ZERO * f256::NEG_ZERO, f256::ZERO);
        assert_eq!(f256::ONE * f256::ZERO, f256::ZERO);
        assert_eq!(f256::ZERO * f256::ONE, f256::ZERO);
        assert_eq!(f256::ONE * f256::NEG_ZERO, f256::NEG_ZERO);
        assert_eq!(f256::NEG_ZERO * f256::ONE, f256::NEG_ZERO);
        assert_eq!(f256::NEG_ONE * f256::ZERO, f256::NEG_ZERO);
        assert_eq!(f256::ZERO * f256::NEG_ONE, f256::NEG_ZERO);
        assert_eq!(f256::NEG_ONE * f256::NEG_ZERO, f256::ZERO);
        assert_eq!(f256::NEG_ZERO * f256::NEG_ONE, f256::ZERO);
    }

    #[test]
    fn test_normal() {
        assert_eq!(f256::ONE * f256::ONE, f256::ONE);
        assert_eq!(f256::ONE * f256::NEG_ONE, f256::NEG_ONE);
        assert_eq!(f256::NEG_ONE * f256::ONE, f256::NEG_ONE);
        assert_eq!(f256::TWO * f256::TWO, f256::from(4.0));
        assert_eq!(f256::from(3.5) * f256::from(2.75), f256::from(9.625));
    }

    #[test]
    fn test_five_pow_102() {
        let five = f256::from(5);
        let five_pow_50 = f256::from(88817841970012523233890533447265625_u128);
        let five_pow_51 = f256::from(444089209850062616169452667236328125_u128);
        let five_pow_101 = five_pow_50 * five_pow_51;
        assert_eq!(
            five_pow_101.as_sign_exp_signif(),
            (
                0,
                0,
                (
                    115912692208981918304116726923363,
                    250037011538279330113129619742442556597
                )
            )
        );
        let five_pow_102 = five_pow_101 * five;
        assert_eq!(
            five_pow_102.as_sign_exp_signif(),
            (
                0,
                0,
                (
                    579563461044909591520583634616818,
                    229337956928581260175524276416908148617
                )
            )
        );
    }

    #[test]
    fn test_subnormal() {
        let x = f256::MIN_GT_ZERO;
        assert_eq!(x * x, f256::ZERO);
        assert_eq!(-x * x, f256::NEG_ZERO);
        assert_eq!(x * -x, f256::NEG_ZERO);
        assert_eq!(-x * -x, f256::NEG_ZERO);
        let y = f256::from(0.1);
        assert_eq!(x * y, f256::ZERO);
        assert_eq!(-x * y, f256::NEG_ZERO);
        assert_eq!(x * -y, f256::NEG_ZERO);
        assert_eq!(-x * -y, f256::NEG_ZERO);
        let y = f256::TWO;
        let z = x + x;
        assert_eq!(x * y, z);
    }

    #[test]
    fn test_overflow() {
        assert_eq!(f256::MAX * f256::TWO, f256::INFINITY);
        assert_eq!(f256::MIN * f256::TWO, f256::NEG_INFINITY);
    }
}
