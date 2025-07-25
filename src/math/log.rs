// ---------------------------------------------------------------------------
// Copyright:   (c) 2024 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use core::{cmp::Ordering, num::FpCategory};

use super::{bkm::bkm_l, Float256, Float512};
use crate::{exp, f256, norm_signif_exp, signif, BigUInt};

pub(crate) fn approx_ln(f: &Float512) -> Float512 {
    // f = m⋅2ᵉ
    // logₙ f = logₙ m + logₙ 2ᵉ = logₙ m + e⋅logₙ 2
    let mut ln = Float512::LN_2 * Float512::from(f.exp());
    ln += &bkm_l(&Float512::from(&f.signif()));
    ln
}

#[inline(always)]
fn ln(x: &f256) -> Float512 {
    debug_assert!(!x.is_special() && x.is_sign_positive());
    approx_ln(&Float512::from(x))
}

impl f256 {
    /// Returns the logarithm of the number with respect to an arbitrary base.
    ///
    /// The result might not be correctly rounded owing to implementation
    /// details; self.log2() can produce more accurate results for base 2, and
    /// self.log10() can produce more accurate results for base 10.
    #[must_use]
    pub fn log(&self, base: &Self) -> Self {
        // logₐ(b) = ln(b) / ln(a)
        match (self.sign(), self.classify()) {
            (_, FpCategory::Zero) => {
                if base.is_special() || base.is_sign_negative() {
                    Self::NAN
                } else {
                    [Self::INFINITY, Self::NEG_INFINITY]
                        [(base >= &Self::ONE) as usize]
                }
            }
            (0, FpCategory::Infinite) => {
                if base.is_special() || base.is_sign_negative() {
                    Self::NAN
                } else {
                    [Self::NEG_INFINITY, Self::INFINITY]
                        [(base >= &Self::ONE) as usize]
                }
            }
            (_, FpCategory::Nan) | (1, _) => Self::NAN,
            _ => {
                // self is finite and > 0
                match (base.sign(), base.classify()) {
                    (_, FpCategory::Zero) => [Self::ZERO, Self::NEG_ZERO]
                        [(self >= &Self::ONE) as usize],
                    (0, FpCategory::Infinite) => [Self::NEG_ZERO, Self::ZERO]
                        [(self >= &Self::ONE) as usize],
                    (_, FpCategory::Nan) | (1, _) => Self::NAN,
                    _ => {
                        // base is finite and > 0
                        if base == &Self::ONE {
                            match self.total_cmp(&Self::ONE) {
                                Ordering::Greater => Self::INFINITY,
                                Ordering::Equal => Self::NAN,
                                Ordering::Less => Self::NEG_INFINITY,
                            }
                        } else {
                            let mut t = ln(self);
                            t /= &ln(base);
                            Self::from(&t)
                        }
                    }
                }
            }
        }
    }

    /// Returns the natural logarithm of the number.
    #[must_use]
    pub fn ln(&self) -> Self {
        // x < 0 or x is nan => ln x is nan
        // x = 0 => ln x = -∞
        // x = ∞ => ln x = ∞
        match (self.sign(), self.classify()) {
            (_, FpCategory::Zero) => Self::NEG_INFINITY,
            (_, FpCategory::Nan) | (1, _) => Self::NAN,
            (0, FpCategory::Infinite) => Self::INFINITY,
            _ => Self::from(&ln(self)),
        }
    }

    /// Returns ln(1+n) (natural logarithm) more accurately than if the
    /// operations were performed separately.
    #[must_use]
    pub fn ln_1p(&self) -> Self {
        // |x| = 0 or x = ∞ => ln (1+x) = x
        if self.eq_zero() || self == &Self::INFINITY {
            return *self;
        }
        // x = -1 => ln 1+x = -∞
        if self == &Self::NEG_ONE {
            return Self::NEG_INFINITY;
        }
        // x < -1 or x is nan => ln 1+x is nan
        if self < &Self::NEG_ONE || self.is_nan() {
            return Self::NAN;
        }
        // x = m⋅2⁻ⁿ⋅2ᵉ with n = 236 and 0 < m⋅2⁻ⁿ < 2
        let e = exp(&self.bits);
        match e {
            ..=-257 => {
                // x < 2⁻²⁵⁶ => ln (1+x) ≈ x-½x² ≈ x
                *self
            }
            // -256..=0 => {
            -256..=492 => {
                // 2⁻²⁵⁶ <= x < 2
                let mut f = Float512::from(self);
                f += &Float512::ONE;
                Self::from(&approx_ln(&f))
            }
            // 1..=492 => {
            //     // 2 <= x < 2⁴⁹³
            //     let m = signif(&self.bits);
            //     let mut m = Float512::new(m.hi.0, m.lo.0, 0_u128, 0_u128);
            //     let mut t = Float512::ONE;
            //     t.ishr(e as u32);
            //     m += &t;
            //     Self::from(&approx_ln(&m, e))
            // }
            _ => {
                // x >= 2⁴⁹³ => ln (1+x) ≈ ln x
                self.ln()
            }
        }
    }

    //noinspection DuplicatedCode
    /// Returns the base 2 logarithm of the number.
    #[must_use]
    pub fn log2(&self) -> Self {
        // x < 0 or x is nan => ln x is nan
        // x = 0 => ln x = -∞
        // x = ∞ => ln x = ∞
        match (self.sign(), self.classify()) {
            (_, FpCategory::Zero) => Self::NEG_INFINITY,
            (_, FpCategory::Nan) | (1, _) => Self::NAN,
            (0, FpCategory::Infinite) => Self::INFINITY,
            _ => {
                // log₂ x = ln x ⋅ log₂ e
                let mut t = ln(self);
                t *= &Float512::LOG2_E;
                Self::from(&t)
            }
        }
    }

    //noinspection DuplicatedCode
    /// Returns the base 10 logarithm of the number.
    #[must_use]
    pub fn log10(&self) -> Self {
        // x < 0 or x is nan => ln x is nan
        // x = 0 => ln x = -∞
        // x = ∞ => ln x = ∞
        match (self.sign(), self.classify()) {
            (_, FpCategory::Zero) => Self::NEG_INFINITY,
            (_, FpCategory::Nan) | (1, _) => Self::NAN,
            (0, FpCategory::Infinite) => Self::INFINITY,
            _ => {
                // log₁₀ x = ln x ⋅ log₁₀ e
                let mut t = ln(self);
                t *= &Float512::LOG10_E;
                Self::from(&t)
            }
        }
    }
}

//noinspection DuplicatedCode
#[cfg(test)]
mod log_tests {
    use super::*;

    #[test]
    fn test_nan() {
        for b in [
            f256::NAN,
            f256::INFINITY,
            f256::NEG_INFINITY,
            f256::from(-1.5),
            f256::from(-1.0),
            f256::from(-0.5),
            f256::from(-0.0),
            f256::from(0.0),
            f256::from(0.5),
            f256::from(1.0),
            f256::from(1.5),
        ] {
            assert!(f256::NAN.log(&b).is_nan());
        }
    }

    #[test]
    fn test_zero() {
        for b in [
            f256::NAN,
            f256::INFINITY,
            f256::NEG_INFINITY,
            f256::from(-1.5),
            f256::from(-1.0),
            f256::from(-0.5),
            f256::from(-0.0),
            f256::from(0.0),
        ] {
            assert!(f256::ZERO.log(&b).is_nan());
            assert!(f256::NEG_ZERO.log(&b).is_nan());
        }
        for b in [
            f256::MIN_GT_ZERO,
            f256::MIN_POSITIVE,
            f256::EPSILON,
            f256::from(1.0) - f256::EPSILON,
        ] {
            assert_eq!(f256::ZERO.log(&b), f256::INFINITY);
            assert_eq!(f256::NEG_ZERO.log(&b), f256::INFINITY);
        }
        for b in [f256::from(1.0), f256::from(1.0) + f256::EPSILON, f256::MAX]
        {
            assert_eq!(f256::ZERO.log(&b), f256::NEG_INFINITY);
            assert_eq!(f256::NEG_ZERO.log(&b), f256::NEG_INFINITY);
        }
    }

    #[test]
    fn test_infinity() {
        for b in [
            f256::NAN,
            f256::INFINITY,
            f256::NEG_INFINITY,
            f256::from(-1.5),
            f256::from(-1.0),
            f256::from(-0.5),
            f256::from(-0.0),
            f256::from(0.0),
        ] {
            assert!(f256::INFINITY.log(&b).is_nan());
        }
        for b in [
            f256::MIN_GT_ZERO,
            f256::MIN_POSITIVE,
            f256::EPSILON,
            f256::from(1.0) - f256::EPSILON,
        ] {
            assert_eq!(f256::INFINITY.log(&b), f256::NEG_INFINITY);
        }
        for b in [
            f256::from(1.0),
            f256::from(1.0) + f256::EPSILON,
            f256::TEN,
            f256::MAX,
        ] {
            assert_eq!(f256::INFINITY.log(&b), f256::INFINITY);
        }
    }

    #[test]
    fn test_neg_values() {
        for a in [
            f256::NEG_INFINITY,
            f256::MIN,
            f256::from(-1.5),
            f256::from(-1.0),
            f256::from(-0.5),
        ] {
            for b in [
                f256::NAN,
                f256::INFINITY,
                f256::NEG_INFINITY,
                f256::from(-1.5),
                f256::from(-1.0),
                f256::from(-0.5),
                f256::from(-0.0),
                f256::from(0.0),
                f256::from(0.5),
                f256::from(1.0),
                f256::from(1.5),
            ] {
                assert!(a.log(&b).is_nan());
            }
        }
    }

    #[test]
    fn test_one() {
        assert_eq!(f256::ONE.log(&f256::INFINITY), f256::ZERO);
        assert_eq!(f256::ONE.log(&f256::from(1.2)), f256::ZERO);
        assert!(f256::ONE.log(&f256::ONE).is_nan());
        assert_eq!(f256::ONE.log(&f256::from(0.5)), f256::NEG_ZERO);
        assert_eq!(f256::ONE.log(&f256::ZERO), f256::NEG_ZERO);
        assert_eq!(f256::ONE.log(&f256::NEG_ZERO), f256::NEG_ZERO);
    }

    #[test]
    fn test_base_nan() {
        for a in [
            f256::NAN,
            f256::INFINITY,
            f256::NEG_INFINITY,
            f256::from(-1.5),
            f256::from(-1.0),
            f256::from(-0.5),
            f256::from(-0.0),
            f256::from(0.0),
            f256::from(0.5),
            f256::from(1.0),
            f256::from(1.5),
        ] {
            assert!(a.log(&f256::NAN).is_nan());
        }
    }

    #[test]
    fn test_base_infinite() {
        for a in [
            f256::NAN,
            f256::INFINITY,
            f256::NEG_INFINITY,
            f256::from(-1.5),
            f256::from(-1.0),
            f256::from(-0.5),
            f256::from(-0.0),
            f256::from(0.0),
        ] {
            assert!(a.log(&f256::INFINITY).is_nan());
        }
        for a in [
            f256::MIN_GT_ZERO,
            f256::MIN_POSITIVE,
            f256::EPSILON,
            f256::from(1.0) - f256::EPSILON,
        ] {
            assert_eq!(a.log(&f256::INFINITY), f256::NEG_ZERO);
        }
        for a in [
            f256::from(1.0),
            f256::from(1.0) + f256::EPSILON,
            f256::TEN,
            f256::MAX,
        ] {
            assert_eq!(a.log(&f256::INFINITY), f256::ZERO);
        }
    }

    #[test]
    fn test_base_zero() {
        for a in [
            f256::NAN,
            f256::INFINITY,
            f256::NEG_INFINITY,
            f256::from(-1.5),
            f256::from(-1.0),
            f256::from(-0.5),
            f256::from(-0.0),
            f256::from(0.0),
        ] {
            assert!(a.log(&f256::ZERO).is_nan());
            assert!(a.log(&f256::NEG_ZERO).is_nan());
        }
        for a in [
            f256::MIN_GT_ZERO,
            f256::MIN_POSITIVE,
            f256::EPSILON,
            f256::from(1.0) - f256::EPSILON,
        ] {
            assert_eq!(a.log(&f256::ZERO), f256::ZERO);
            assert_eq!(a.log(&f256::NEG_ZERO), f256::ZERO);
        }
        for a in [
            f256::from(1.0),
            f256::from(1.0) + f256::EPSILON,
            f256::TEN,
            f256::MAX,
        ] {
            assert_eq!(a.log(&f256::ZERO), f256::NEG_ZERO);
            assert_eq!(a.log(&f256::NEG_ZERO), f256::NEG_ZERO);
        }
    }

    #[test]
    fn test_neg_base() {
        for b in [
            f256::NEG_INFINITY,
            f256::MIN,
            f256::from(-1.5),
            f256::from(-1.0),
            f256::from(-0.5),
        ] {
            for a in [
                f256::NAN,
                f256::INFINITY,
                f256::NEG_INFINITY,
                f256::from(-1.5),
                f256::from(-1.0),
                f256::from(-0.5),
                f256::from(-0.0),
                f256::from(0.0),
                f256::from(0.5),
                f256::from(1.0),
                f256::from(1.5),
            ] {
                assert!(a.log(&b).is_nan());
            }
        }
    }

    #[test]
    fn test_base_one() {
        assert_eq!(f256::INFINITY.log(&f256::ONE), f256::INFINITY);
        assert_eq!(f256::from(1.2).log(&f256::ONE), f256::INFINITY);
        assert!(f256::ONE.log(&f256::ONE).is_nan());
        assert_eq!(f256::from(0.5).log(&f256::ONE), f256::NEG_INFINITY);
        assert_eq!(f256::ZERO.log(&f256::ONE), f256::NEG_INFINITY);
        assert_eq!(f256::NEG_ZERO.log(&f256::ONE), f256::NEG_INFINITY);
    }

    #[test]
    fn test_near_one() {
        let x = f256::ONE - f256::EPSILON;
        let y = f256::ONE + f256::EPSILON;
        assert_eq!(x.log(&y), -y);
        assert_eq!(y.log(&x), -x);
    }
}

#[cfg(test)]
mod ln_tests {
    use core::str::FromStr;

    use super::*;
    use crate::consts::E;

    #[test]
    fn test_undefined() {
        assert!(f256::NEG_ONE.ln().is_nan());
        assert!(f256::NEG_INFINITY.ln().is_nan());
        assert!(f256::NAN.ln().is_nan());
    }

    #[test]
    fn test_specials() {
        assert_eq!(f256::ZERO.ln(), f256::NEG_INFINITY);
        assert_eq!(f256::NEG_ZERO.ln(), f256::NEG_INFINITY);
        assert_eq!(f256::INFINITY.ln(), f256::INFINITY);
    }

    #[test]
    fn test_ln_1() {
        assert_eq!(f256::ONE.ln(), f256::ZERO);
    }

    #[test]
    fn test_ln_1_plus_epsilon() {
        let x = f256::ONE + f256::EPSILON;
        let s =
            "9.0556790788267123675091192908877917806825311981391381895826\
                    1488993550128e-72";
        let ln_x = f256::from_str(s).unwrap();
        assert_eq!(x.ln(), ln_x);
    }

    #[test]
    fn test_ln_1_plus_7_times_epsilon() {
        let x = f256::ONE + (f256::from(7) * f256::EPSILON);
        let s =
            "6.3389753551786986572563835036214542464777718386973967327078\
                    3042295485073e-71";
        let ln_x = f256::from_str(s).unwrap();
        assert_eq!(x.ln(), ln_x);
    }

    #[test]
    fn test_ln_1_minus_epsilon() {
        let x = f256::ONE - f256::EPSILON;
        assert_eq!(x.ln(), -f256::EPSILON);
    }

    #[test]
    fn test_ln_epsilon() {
        let s =
            "-163.5827346121470930224667806641296700658180317090202399724\
                    80482240440895";
        let ln_eps = f256::from_str(s).unwrap();
        assert_eq!(f256::EPSILON.ln(), ln_eps);
    }

    #[test]
    fn test_ln_min_positive() {
        let s =
            "-1.817029882063451833012520627832893219084477562214660328257\
                    03299048617191e5";
        let ln_mp = f256::from_str(s).unwrap();
        assert_eq!(f256::MIN_POSITIVE.ln(), ln_mp);
    }

    #[test]
    fn test_ln_5_times_min_positive() {
        let x = f256::from(5) * f256::MIN_POSITIVE;
        let s =
            "-1.817013787684327492008774620239560957208082306201117643079\
                    81386400725716e5";
        let ln_5mp = f256::from_str(s).unwrap();
        assert_eq!(x.ln(), ln_5mp);
    }

    #[test]
    fn test_ln_11_times_min_gt_zero() {
        let x = f256::from(11) * f256::MIN_GT_ZERO;
        let s =
            "-1.818641730456845320237304676203754864492137525463211156485\
                    00560963148501e5";
        let ln_11gt0 = f256::from_str(s).unwrap();
        assert_eq!(x.ln(), ln_11gt0);
    }

    #[test]
    fn test_ln_e() {
        assert_eq!(E.ln(), f256::ONE);
    }

    #[test]
    fn test_ln_max() {
        let s =
            "1.8170437450070630319187089724753223826158390722173475333621\
                    1540408636177e5";
        let ln_max = f256::from_str(s).unwrap();
        assert_eq!(f256::MAX.ln(), ln_max);
    }
}

#[cfg(test)]
mod ln_1p_tests {
    use core::{ops::Neg, str::FromStr};

    use super::*;
    use crate::consts::E;

    #[test]
    fn test_undefined() {
        assert!(f256::TWO.neg().ln_1p().is_nan());
        assert!(f256::NEG_INFINITY.ln_1p().is_nan());
        assert!(f256::NAN.ln_1p().is_nan());
    }

    #[test]
    fn test_specials() {
        assert_eq!(f256::NEG_ONE.ln_1p(), f256::NEG_INFINITY);
        assert_eq!(f256::ZERO.ln_1p(), f256::ZERO);
        assert_eq!(f256::NEG_ZERO.ln_1p(), f256::NEG_ZERO);
        assert_eq!(f256::INFINITY.ln_1p(), f256::INFINITY);
    }

    #[test]
    fn test_ln_near_1() {
        assert_eq!(f256::ONE.ln(), f256::ZERO);
        let f = f256::ONE - f256::EPSILON;
        assert_eq!(f.ln(), -f256::EPSILON);
        let f = f256::ONE + f256::EPSILON;
        assert_eq!(f.ln(), f256::EPSILON - f256::EPSILON.ulp().div2());
    }

    #[test]
    fn test_ln_1p_0() {
        assert_eq!(f256::ZERO.ln_1p(), f256::ZERO);
    }

    #[test]
    fn test_ln_1p_1() {
        assert_eq!(f256::ONE.ln_1p(), f256::from(&Float512::LN_2));
    }

    #[test]
    fn test_ln_1p_epsilon() {
        let x = f256::EPSILON;
        let s =
            "9.0556790788267123675091192908877917806825311981391381895826\
                    1488993550128e-72";
        let ln_1p_x = f256::from_str(s).unwrap();
        assert_eq!(x.ln_1p(), ln_1p_x);
    }

    #[test]
    fn test_ln_1p_7_times_epsilon() {
        let x = f256::from(7) * f256::EPSILON;
        let s =
            "6.3389753551786986572563835036214542464777718386973967327078\
                    3042295485073e-71";
        let ln_1p_x = f256::from_str(s).unwrap();
        assert_eq!(x.ln_1p(), ln_1p_x);
    }

    #[test]
    fn test_ln_1p_e() {
        assert_eq!(E.ln_1p(), (E + f256::ONE).ln());
    }

    #[test]
    fn test_ln_1p_some_gte() {
        let s =
            "3.9172112277498476971618994396640379622838972944553968700466\
                    109220590303";
        let x = f256::from_str(s).unwrap();
        let s =
            "1.5927415461708937796028041885177758718526978971737978008094\
                    3200179146619";
        let ln_1p_x = f256::from_str(s).unwrap();
        assert_eq!(x.ln_1p(), ln_1p_x);
    }

    #[test]
    fn test_ln_1p_some_large_value() {
        let s =
            "6.4925516411401714605181493976904708352003549751890256895138\
                    1816355622497e69";
        let x = f256::from_str(s).unwrap();
        let s =
            "160.74902703508073753030346151423598221171734146783516372812\
                    6476793486362";
        let ln_1p_x = f256::from_str(s).unwrap();
        assert_eq!(x.ln_1p(), ln_1p_x);
    }

    #[test]
    fn test_ln_1p_2_pow_230() {
        let x = f256::from(2_f64.powi(230));
        let s =
            "159.42385152878742116596338793538061065736503090285870844775\
                    6402183480534";
        let ln_1p_x = f256::from_str(s).unwrap();
        assert_eq!(x.ln_1p(), ln_1p_x);
    }

    #[test]
    fn test_ln_1p_2_pow_231() {
        let x = f256::from(2_f64.powi(231));
        assert_eq!(x.ln_1p(), x.ln());
    }

    #[test]
    fn test_ln_1p_2_pow_minus_237() {
        let x = f256::from(2_f64.powi(-237));
        assert_eq!(x.ln_1p(), x);
    }
}

#[cfg(test)]
mod log2_tests {
    use super::*;

    #[test]
    fn test_undefined() {
        assert!(f256::NEG_ONE.log2().is_nan());
        assert!(f256::NEG_INFINITY.log2().is_nan());
        assert!(f256::NAN.log2().is_nan());
    }

    #[test]
    fn test_specials() {
        assert_eq!(f256::ZERO.log2(), f256::NEG_INFINITY);
        assert_eq!(f256::NEG_ZERO.log2(), f256::NEG_INFINITY);
        assert_eq!(f256::INFINITY.log2(), f256::INFINITY);
    }

    #[test]
    fn test_log2_1() {
        assert_eq!(f256::ONE.log2(), f256::ZERO);
    }
}

#[cfg(test)]
mod log10_tests {
    use super::*;

    #[test]
    fn test_undefined() {
        assert!(f256::NEG_ONE.log10().is_nan());
        assert!(f256::NEG_INFINITY.log10().is_nan());
        assert!(f256::NAN.log10().is_nan());
    }

    #[test]
    fn test_specials() {
        assert_eq!(f256::ZERO.log10(), f256::NEG_INFINITY);
        assert_eq!(f256::NEG_ZERO.log10(), f256::NEG_INFINITY);
        assert_eq!(f256::INFINITY.log10(), f256::INFINITY);
    }

    #[test]
    fn test_log10_1() {
        assert_eq!(f256::ONE.log10(), f256::ZERO);
    }
}
