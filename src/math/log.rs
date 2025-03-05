// ---------------------------------------------------------------------------
// Copyright:   (c) 2024 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use core::{cmp::Ordering, num::FpCategory};

use super::{bkm::bkm_l, BigFloat, FP492};
use crate::{exp, f256, norm_signif_exp, signif, BigUInt};

// LN_2 = ◯₄₉₂(logₑ(2)) =
// 0.693147180559945309417232121458176568075500134360255254120680009493393621\
// 96969471560586332699641868754200148102057068573368552023575813055703267075\
// 16657753
pub(crate) const LN_2: FP492 = FP492::new(
    0x00000b17217f7d1cf79abc9e3b39803f,
    0x2f6af40f343267298b62d8a0d175b8ba,
    0xafa2be7b876206debac98559552fb4af,
    0xa1b10ed2eae35c138214427573b29117,
);
// LN_10 = ◯₄₉₂(logₑ(10)) =
// 2.302585092994045684017991454684364207601101488628772976033327900967572609\
// 67735248023599720508959829834196778404228624863340952546508280675666628736\
// 9077486
pub(crate) const LN_10: FP492 = FP492::new(
    0x000024d763776aaa2b05ba95b58ae0b4,
    0xc28a38a3fb3e76977e43a0f187a0807c,
    0x0b5ca58bc0b5ec6a0417331c32f00b17,
    0xc35a0b1889061042f8b6bee3de2100b9,
);
// LOG2_E = ◯₄₉₂(log₂(e)) =
// 1.442695040888963407359924681001892137426645954152985934135449406931109219\
// 18118507988552662289350634449699751830965254425559310168716835964272066215\
// 82310518
pub(crate) const LOG2_E: FP492 = FP492::new(
    0x0000171547652b82fe1777d0ffda0d23,
    0xa7d11d6aef551bad2b4b1164a2cd9a34,
    0x2648fbc3887eeaa2ed9ac49b25eeb82d,
    0x7c167d52173cc1895213f897f5e06a7c,
);
// LOG10_E = ◯₄₉₂(log₁₀(e)) =
// 0.434294481903251827651128918916605082294397005803666566114453783165864649\
// 20887077472922494933843174831870610674476630373364167928715896390656922106\
// 466854313
pub(crate) const LOG10_E: FP492 = FP492::new(
    0x000006f2dec549b9438ca9aadd557d69,
    0x9ee191f71a30122e4d1011d1f96a27bc,
    0x7529e3aa1277d0a0179f94911aac9632,
    0x3250a8c671decfe9c6e5e37d15c69646,
);

#[inline(always)]
fn approx_ln(m: &FP492, e: i32) -> FP492 {
    debug_assert!(m >= &FP492::ONE);
    let ln_m = bkm_l(&m);
    let mut ln = LN_2;
    ln *= &FP492::from(e);
    ln += &ln_m;
    ln
}

fn ln(x: &f256) -> FP492 {
    debug_assert!(!x.is_special() && x.is_sign_positive());
    let (m, e) = norm_signif_exp(&x.bits);
    // x = m⋅2⁻ⁿ⋅2ᵉ with n = 236 and 1 <= m⋅2⁻ⁿ < 2
    // By using m as the hi-part of an FP492 value,
    // we turn m⋅2⁻ⁿ into m', so that
    // x = m'⋅2ᵉ
    // ln x = ln (m'⋅2ᵉ) = ln m' + ln 2ᵉ = ln (m') + e⋅ln 2
    let m = FP492::new(m.hi.0, m.lo.0, 0_u128, 0_u128);
    approx_ln(&m, e)
}

impl f256 {
    /// Returns the logarithm of the number with respect to an arbitrary base.
    ///
    /// The result might not be correctly rounded owing to implementation
    /// details; self.log2() can produce more accurate results for base 2, and
    /// self.log10() can produce more accurate results for base 10.
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
            (_, FpCategory::Nan) => Self::NAN,
            (1, _) => Self::NAN,
            _ => {
                // self is finite and > 0
                match (base.sign(), base.classify()) {
                    (_, FpCategory::Zero) => [Self::ZERO, Self::NEG_ZERO]
                        [(self >= &Self::ONE) as usize],
                    (0, FpCategory::Infinite) => [Self::NEG_ZERO, Self::ZERO]
                        [(self >= &Self::ONE) as usize],
                    (_, FpCategory::Nan) => Self::NAN,
                    (1, _) => Self::NAN,
                    _ => {
                        // base is finite and > 0
                        if base == &Self::ONE {
                            match self.total_cmp(&Self::ONE) {
                                Ordering::Greater => Self::INFINITY,
                                Ordering::Equal => Self::NAN,
                                Ordering::Less => Self::NEG_INFINITY,
                            }
                        } else {
                            let mut t = BigFloat::from(&ln(self));
                            t /= &BigFloat::from(&ln(base));
                            Self::from(&t)
                        }
                    }
                }
            }
        }
    }

    /// Returns the natural logarithm of the number.
    pub fn ln(&self) -> Self {
        // x < 0 or x is nan => ln x is nan
        // x = 0 => ln x = -∞
        // x = ∞ => ln x = ∞
        match (self.sign(), self.classify()) {
            (_, FpCategory::Zero) => Self::NEG_INFINITY,
            (_, FpCategory::Nan) => Self::NAN,
            (0, FpCategory::Infinite) => Self::INFINITY,
            (1, _) => Self::NAN,
            _ => Self::from(&ln(self)),
        }
    }

    /// Returns ln(1+n) (natural logarithm) more accurately than if the
    /// operations were performed separately.
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
            -256..=0 => {
                // 2⁻²⁵⁶ <= x < 2
                let mut m = FP492::from(self);
                m += &FP492::ONE;
                Self::from(&approx_ln(&m, 0))
            }
            1..=492 => {
                // 2 <= x < 2⁴⁹³
                let m = signif(&self.bits);
                let mut m = FP492::new(m.hi.0, m.lo.0, 0_u128, 0_u128);
                let mut t = FP492::ONE;
                t.ishr(e as u32);
                m += &t;
                Self::from(&approx_ln(&m, e))
            }
            _ => {
                // x >= 2⁴⁹³ => ln (1+x) ≈ ln x
                self.ln()
            }
        }
    }

    //noinspection DuplicatedCode
    /// Returns the base 2 logarithm of the number.
    pub fn log2(&self) -> Self {
        // x < 0 or x is nan => ln x is nan
        // x = 0 => ln x = -∞
        // x = ∞ => ln x = ∞
        match (self.sign(), self.classify()) {
            (_, FpCategory::Zero) => Self::NEG_INFINITY,
            (_, FpCategory::Nan) => Self::NAN,
            (0, FpCategory::Infinite) => Self::INFINITY,
            (1, _) => Self::NAN,
            _ => {
                // log₂ x = ln x ⋅ log₂ e
                let mut t = ln(&self);
                t *= &LOG2_E;
                Self::from(&t)
            }
        }
    }

    //noinspection DuplicatedCode
    /// Returns the base 10 logarithm of the number.
    pub fn log10(&self) -> Self {
        // x < 0 or x is nan => ln x is nan
        // x = 0 => ln x = -∞
        // x = ∞ => ln x = ∞
        match (self.sign(), self.classify()) {
            (_, FpCategory::Zero) => Self::NEG_INFINITY,
            (_, FpCategory::Nan) => Self::NAN,
            (0, FpCategory::Infinite) => Self::INFINITY,
            (1, _) => Self::NAN,
            _ => {
                // log₁₀ x = ln x ⋅ log₁₀ e
                let mut t = ln(&self);
                t *= &LOG10_E;
                Self::from(&t)
            }
        }
    }
}

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
        assert_eq!(f.ln(), f256::EPSILON - f256::EPSILON.ulp() / f256::TWO);
    }

    #[test]
    fn test_ln_1p_0() {
        assert_eq!(f256::ZERO.ln_1p(), f256::ZERO);
    }

    #[test]
    fn test_ln_1p_1() {
        assert_eq!(f256::ONE.ln_1p(), f256::from(&LN_2));
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
