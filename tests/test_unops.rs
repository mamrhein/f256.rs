// ---------------------------------------------------------------------------
// Copyright:   (c) 2022 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

#[cfg(test)]
mod min_max_tests {
    use ::f256::f256;

    #[test]
    fn test_min_nan() {
        assert_eq!(f256::NAN.min(f256::ONE), f256::ONE);
        assert_eq!(f256::ONE.min(f256::NAN), f256::ONE);
    }

    #[test]
    fn test_max_nan() {
        assert_eq!(f256::NAN.max(f256::ONE), f256::ONE);
        assert_eq!(f256::ONE.max(f256::NAN), f256::ONE);
    }
}

#[cfg(test)]
mod abs_tests {
    use ::f256::f256;

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
    use core::ops::Neg;

    use ::f256::f256;

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
        assert_eq!(f.neg().trunc(), -f);
        assert_eq!(g.trunc(), f);
        assert_eq!(h.trunc(), -f);
    }

    #[test]
    fn test_lt_1() {
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
    use ::f256::f256;

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
    fn test_lt_1() {
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

#[cfg(test)]
mod ceil_tests {
    use core::ops::Neg;

    use ::f256::f256;

    #[test]
    fn test_nan() {
        assert!(f256::NAN.ceil().is_nan());
        assert!((-f256::NAN).ceil().is_nan());
    }

    #[test]
    fn test_inf() {
        assert_eq!(f256::INFINITY.ceil(), f256::INFINITY);
        assert_eq!(f256::NEG_INFINITY.ceil(), f256::NEG_INFINITY);
    }

    #[test]
    fn test_zero() {
        assert_eq!(f256::ZERO.ceil(), f256::ZERO);
        assert_eq!(f256::NEG_ZERO.ceil(), f256::NEG_ZERO);
    }

    #[test]
    fn test_normal() {
        let f = f256::from(18);
        let g = f256::from(17.625_f64);
        let h = -g;
        assert_eq!(f.ceil(), f);
        assert_eq!(f.neg().ceil(), -f);
        assert_eq!(g.ceil(), f);
        assert_eq!(h.ceil(), f256::ONE - f);
    }

    #[test]
    fn test_lt_0() {
        let f = f256::from(0.99999_f64);
        assert_eq!(f.ceil(), f256::ONE);
        let e = f256::EPSILON;
        assert_eq!(e.ceil(), f256::ONE);
    }

    #[test]
    fn test_gt_2_pow_237() {
        let f = f256::from(1.3097428e71_f64);
        assert_eq!(f.ceil(), f);
    }

    #[test]
    fn test_subnormal() {
        let f = f256::MIN_GT_ZERO;
        assert_eq!(f.ceil(), f256::ONE);
        let g = -f;
        assert_eq!(g.ceil(), f256::ZERO);
    }
}

#[cfg(test)]
mod floor_tests {
    use core::ops::Neg;

    use ::f256::f256;

    #[test]
    fn test_nan() {
        assert!(f256::NAN.floor().is_nan());
        assert!((-f256::NAN).floor().is_nan());
    }

    #[test]
    fn test_inf() {
        assert_eq!(f256::INFINITY.floor(), f256::INFINITY);
        assert_eq!(f256::NEG_INFINITY.floor(), f256::NEG_INFINITY);
    }

    #[test]
    fn test_zero() {
        assert_eq!(f256::ZERO.floor(), f256::ZERO);
        assert_eq!(f256::NEG_ZERO.floor(), f256::NEG_ZERO);
    }

    #[test]
    fn test_normal() {
        let f = f256::from(28);
        let g = f256::from(28.025_f64);
        let h = -g;
        assert_eq!(f.floor(), f);
        assert_eq!(f.neg().floor(), -f);
        assert_eq!(g.floor(), f);
        assert_eq!(h.floor(), -f - f256::ONE);
    }

    #[test]
    fn test_lt_0() {
        let f = f256::from(-0.99999_f64);
        assert_eq!(f.floor(), f256::NEG_ONE);
        let e = f256::EPSILON;
        assert_eq!(e.floor(), f256::ZERO);
    }

    #[test]
    fn test_gt_2_pow_237() {
        let f = f256::from(1.3097428e71_f64);
        assert_eq!(f.floor(), f);
    }

    #[test]
    fn test_subnormal() {
        let f = f256::MIN_GT_ZERO;
        assert_eq!(f.floor(), f256::ZERO);
        let g = -f;
        assert_eq!(g.floor(), f256::NEG_ONE);
    }
}

#[cfg(test)]
mod round_tests {
    use core::ops::Neg;

    use ::f256::f256;

    #[test]
    fn test_nan() {
        assert!(f256::NAN.round().is_nan());
        assert!((-f256::NAN).round().is_nan());
        assert!(f256::NAN.round_tie_even().is_nan());
        assert!((-f256::NAN).round_tie_even().is_nan());
    }

    #[test]
    fn test_inf() {
        assert_eq!(f256::INFINITY.round(), f256::INFINITY);
        assert_eq!(f256::NEG_INFINITY.round(), f256::NEG_INFINITY);
        assert_eq!(f256::INFINITY.round_tie_even(), f256::INFINITY);
        assert_eq!(f256::NEG_INFINITY.round_tie_even(), f256::NEG_INFINITY);
    }

    #[test]
    fn test_zero() {
        assert_eq!(f256::ZERO.round(), f256::ZERO);
        assert_eq!(f256::NEG_ZERO.round(), f256::NEG_ZERO);
        assert_eq!(f256::ZERO.round_tie_even(), f256::ZERO);
        assert_eq!(f256::NEG_ZERO.round_tie_even(), f256::NEG_ZERO);
    }

    #[test]
    fn test_normal_down() {
        let f = f256::from(28);
        let g = f256::from(28.025_f64);
        let h = -g;
        assert_eq!(f.round(), f);
        assert_eq!(f.neg().round(), -f);
        assert_eq!(g.round(), f);
        assert_eq!(h.round(), -f);
        assert_eq!(f.round_tie_even(), f);
        assert_eq!(f.neg().round_tie_even(), -f);
        assert_eq!(g.round_tie_even(), f);
        assert_eq!(h.round_tie_even(), -f);
    }

    #[test]
    fn test_normal_half_up() {
        let f = f256::from(3);
        let g = f256::from(2.5_f64);
        let h = f256::from(-2.5_f64);
        assert_eq!(f.round(), f);
        assert_eq!(f.neg().round(), -f);
        assert_eq!(g.round(), f);
        assert_eq!(h.round(), -f);
    }

    #[test]
    fn test_normal_half_even() {
        let f = f256::from(2);
        let g = f256::from(2.5_f64);
        let h = f256::from(-2.5_f64);
        assert_eq!(f.round_tie_even(), f);
        assert_eq!(f.neg().round_tie_even(), -f);
        assert_eq!(g.round_tie_even(), f);
        assert_eq!(h.round_tie_even(), -f);
    }

    #[test]
    fn test_normal_up() {
        let f = f256::from(28);
        let g = f256::from(27.725_f64);
        let h = -g;
        assert_eq!(f.round(), f);
        assert_eq!(g.round(), f);
        assert_eq!(h.round(), -f);
        assert_eq!(f.round_tie_even(), f);
        assert_eq!(g.round_tie_even(), f);
        assert_eq!(h.round_tie_even(), -f);
    }

    #[test]
    fn test_lt_0() {
        let f = f256::from(-0.99999_f64);
        assert_eq!(f.round(), f256::NEG_ONE);
        let e = f256::EPSILON;
        assert_eq!(e.round(), f256::ZERO);
        let g = f256::from(0.5_f64) - e;
        assert_eq!(g.round(), f256::ZERO);
        assert_eq!(g.round_tie_even(), f256::ZERO);
        let h = f256::from(0.5_f64);
        assert_eq!(h.round(), f256::ONE);
        assert_eq!(h.round_tie_even(), f256::ZERO);
    }

    #[test]
    fn test_gt_2_pow_237() {
        let f = f256::from(1.3097428e71_f64);
        assert_eq!(f.round(), f);
        assert_eq!(f.round_tie_even(), f);
    }

    #[test]
    fn test_subnormal() {
        let f = f256::MIN_GT_ZERO;
        assert_eq!(f.round(), f256::ZERO);
        let g = -f;
        assert_eq!(g.round(), f256::ZERO);
    }
}

#[cfg(test)]
mod rad_degree_tests {
    use ::f256::{
        consts::{FRAC_PI_2, FRAC_PI_3, PI},
        f256,
    };

    #[test]
    fn test_to_degrees() {
        let d180 = f256::from(180);
        let d60 = f256::from(60);
        let x = f256::ONE.to_degrees();
        assert_eq!(
            x.as_sign_exp_signif(),
            (
                0,
                -228,
                (
                    72631029290303093375423554125059,
                    97528794516718399009463026371013945029
                )
            )
        );
        let x = PI.to_degrees();
        assert_eq!(x, d180);
        let x = (f256::TWO * PI).to_degrees();
        assert_eq!(x, f256::TWO * d180);
        let x = FRAC_PI_3.to_degrees();
        // Here we have a rounding error of 2⁻²³¹
        assert_eq!(x - d60, f256::from_sign_exp_signif(0, -231, (0, 1)));
    }

    #[test]
    fn test_to_radians() {
        let d180 = f256::from(180);
        let d90 = f256::from(90);
        let x = f256::ONE.to_radians();
        assert_eq!(
            x.as_sign_exp_signif(),
            (
                0,
                -241,
                (
                    181245351844781960018527624030536,
                    122691704046858638295272262152299957397
                )
            )
        );
        let x = d180.to_radians();
        assert_eq!(x, PI);
        let x = d90.to_radians();
        assert_eq!(x, FRAC_PI_2);
    }
}
