// ---------------------------------------------------------------------------
// Copyright:   (c) 2024 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use super::{feynman, BigFloat, FP509};
use crate::{exp, f256, norm_signif, BigUInt};

// LN_2 = ◯₂₅₅(ln(2)) =
// 0.6931471805599453094172321214581765680755001343602552541206800094933936219697
pub(crate) const LN_2: BigFloat = BigFloat::new(
    1,
    -1,
    (
        0x58b90bfbe8e7bcd5e4f1d9cc01f97b57,
        0xa079a193394c5b16c5068badc5d57d16,
    ),
);
// LN_10 = ◯₂₅₅(ln(10)) =
// 2.3025850929940456840179914546843642076011014886287729760333279009675726096773
pub(crate) const LN_10: BigFloat = BigFloat::new(
    1,
    1,
    (
        0x49aec6eed554560b752b6b15c1698514,
        0x7147f67ced2efc8741e30f4100f816b9,
    ),
);
// LOG2_E = ◯₂₅₅(log2(e)) =
// 1.4426950408889634073599246810018921374266459541529859341354494069311092191812
pub(crate) const LOG2_E: BigFloat = BigFloat::new(
    1,
    0,
    (
        0x5c551d94ae0bf85ddf43ff68348e9f44,
        0x75abbd546eb4ad2c45928b3668d09924,
    ),
);
// LOG10_E = ◯₂₅₅(log10(e)) =
// 0.43429448190325182765112891891660508229439700580366656611445378316586464920887
pub(crate) const LOG10_E: BigFloat = BigFloat::new(
    1,
    -2,
    (
        0x6f2dec549b9438ca9aadd557d699ee19,
        0x1f71a30122e4d1011d1f96a27bc7529e,
    ),
);

fn ln(x: &f256) -> BigFloat {
    debug_assert!(!x.is_special() && x.is_sign_positive());
    let (mut m, sh) = norm_signif(&x.bits);
    let e = exp(&x.bits) - sh as i32;
    // x = m⋅2⁻ⁿ⋅2ᵉ with n = 236 and 1 < m⋅2⁻ⁿ < 2
    // m has 19 leading zeroes. By using it as the hi-part of an FP509 value,
    // we turn m⋅2⁻ⁿ into m⋅2⁻ⁿ⁻¹⁸, so that
    // x = m⋅2⁻ⁿ⁻¹⁸⋅2ᵉ⁺¹⁸
    // ln x = ln (m⋅2⁻ⁿ⁻¹⁸) + ln 2ᵉ⁺¹⁸ = ln (m⋅2⁻ⁿ) + (e+18)⋅ln 2
    // println!("  x: {:?}", FP509::from(&x.significand()));
    m <<= 17;
    let m = FP509::new(m.hi.0, m.lo.0, 0_u128, 0_u128);
    let ln_m = feynman::feynman(&m);
    let mut ln = LN_2;
    ln *= &BigFloat::from(e);
    ln += &BigFloat::from(&ln_m);
    ln
}

impl f256 {
    /// Returns the logarithm of the number with respect to an arbitrary base.
    ///
    /// The result might not be correctly rounded owing to implementation
    /// details; self.log2() can produce more accurate results for base 2, and
    /// self.log10() can produce more accurate results for base 10.
    pub fn log(self, base: Self) -> Self {
        unimplemented!()
    }

    /// Returns the natural logarithm of the number.
    pub fn ln(&self) -> Self {
        // x <= 0 or x is infinite or nan => ln x is nan
        if self.is_special() || self.is_sign_negative() {
            return Self::NAN;
        }
        Self::from(&ln(self))
    }

    /// Returns ln(1+n) (natural logarithm) more accurately than if the
    /// operations were performed separately.
    pub fn ln_1p(&self) -> Self {
        unimplemented!()
    }

    /// Returns the base 2 logarithm of the number.
    pub fn log2(&self) -> Self {
        // x <= 0 or x is infinite or nan => log₂ x is nan
        if self.is_special() || self.is_sign_negative() {
            return Self::NAN;
        }
        // log₂ x = ln x ⋅ log₂ e
        let mut t = ln(&self) * &LOG2_E;
        Self::from(&t)
    }

    /// Returns the base 10 logarithm of the number.
    pub fn log10(&self) -> Self {
        // x <= 0 or x is infinite or nan => log₁₀ x is nan
        if self.is_special() || self.is_sign_negative() {
            return Self::NAN;
        }
        // log₁₀ x = ln x ⋅ log₁₀ e
        let mut t = ln(&self) * &LOG10_E;
        Self::from(&t)
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
        assert!(f256::INFINITY.ln().is_nan());
        assert!(f256::NEG_INFINITY.ln().is_nan());
        assert!(f256::NAN.ln().is_nan());
    }

    #[test]
    fn test_ln_1() {
        assert_eq!(f256::ONE.ln(), f256::ZERO);
    }

    #[test]
    fn test_ln_1_plus_epsilon() {
        let x = f256::ONE + f256::EPSILON;
        let s = "9.0556790788267123675091192908877917806825311981391381895826\
                    1488993550128e-72";
        let ln_x = f256::from_str(s).unwrap();
        assert_eq!(x.ln(), ln_x);
    }

    #[test]
    fn test_ln_1_plus_7_times_epsilon() {
        let x = f256::ONE + (f256::from(7) * f256::EPSILON);
        let s = "6.3389753551786986572563835036214542464777718386973967327078\
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
        let s = "-163.5827346121470930224667806641296700658180317090202399724\
                    80482240440895";
        let ln_eps = f256::from_str(s).unwrap();
        assert_eq!(f256::EPSILON.ln(), ln_eps);
    }

    #[test]
    fn test_ln_min_positive() {
        let s = "-1.817029882063451833012520627832893219084477562214660328257\
                    03299048617191e5";
        let ln_mp = f256::from_str(s).unwrap();
        assert_eq!(f256::MIN_POSITIVE.ln(), ln_mp);
    }

    #[test]
    fn test_ln_5_times_min_positive() {
        let x = f256::from(5) * f256::MIN_POSITIVE;
        let s = "-1.817013787684327492008774620239560957208082306201117643079\
                    81386400725716e5";
        let ln_5mp = f256::from_str(s).unwrap();
        assert_eq!(x.ln(), ln_5mp);
    }

    #[test]
    fn test_ln_11_times_min_gt_zero() {
        let x = f256::from(11) * f256::MIN_GT_ZERO;
        let s = "-1.818641730456845320237304676203754864492137525463211156485\
                    00560963148501e5";
        let ln_11gt0 = f256::from_str(s).unwrap();
        assert_eq!(x.ln(), ln_11gt0);
    }

    #[test]
    fn test_ln_e() {
        assert_eq!(E.ln(), f256::ONE);
    }
}

#[cfg(test)]
mod log2_tests {
    use super::*;

    #[test]
    fn test_undefined() {
        assert!(f256::NEG_ONE.log2().is_nan());
        assert!(f256::INFINITY.log2().is_nan());
        assert!(f256::NEG_INFINITY.log2().is_nan());
        assert!(f256::NAN.log2().is_nan());
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
        assert!(f256::INFINITY.log10().is_nan());
        assert!(f256::NEG_INFINITY.log10().is_nan());
        assert!(f256::NAN.log10().is_nan());
    }

    #[test]
    fn test_log10_1() {
        assert_eq!(f256::ONE.log10(), f256::ZERO);
    }
}
