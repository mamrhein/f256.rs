// ---------------------------------------------------------------------------
// Copyright:   (c) 2024 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use super::{feynman, BigFloat, FP492};
use crate::{exp, f256, norm_signif, BigUInt};

// LN_2 = ◯₄₉₂(ln(2)) =
// 0.693147180559945309417232121458176568075500134360255254120680009493393621\
// 96969471560586332699641868754200148102057068573368552023575813055703267075\
// 16657753
pub(crate) const LN_2: FP492 = FP492::new(
    0x00000b17217f7d1cf79abc9e3b39803f,
    0x2f6af40f343267298b62d8a0d175b8ba,
    0xafa2be7b876206debac98559552fb4af,
    0xa1b10ed2eae35c138214427573b29117,
);
// LN_10 = ◯₄₉₂(ln(10)) =
// 2.302585092994045684017991454684364207601101488628772976033327900967572609\
// 67735248023599720508959829834196778404228624863340952546508280675666628736\
// 9077486
pub(crate) const LN_10: FP492 = FP492::new(
    0x000024d763776aaa2b05ba95b58ae0b4,
    0xc28a38a3fb3e76977e43a0f187a0807c,
    0x0b5ca58bc0b5ec6a0417331c32f00b17,
    0xc35a0b1889061042f8b6bee3de2100b9,
);
// LOG2_E = ◯₄₉₂(log2(e)) =
// 1.442695040888963407359924681001892137426645954152985934135449406931109219\
// 18118507988552662289350634449699751830965254425559310168716835964272066215\
// 82310518
pub(crate) const LOG2_E: FP492 = FP492::new(
    0x0000171547652b82fe1777d0ffda0d23,
    0xa7d11d6aef551bad2b4b1164a2cd9a34,
    0x2648fbc3887eeaa2ed9ac49b25eeb82d,
    0x7c167d52173cc1895213f897f5e06a7c,
);
// LOG10_E = ◯₄₉₂(log10(e)) =
// 0.434294481903251827651128918916605082294397005803666566114453783165864649\
// 20887077472922494933843174831870610674476630373364167928715896390656922106\
// 466854313
pub(crate) const LOG10_E: FP492 = FP492::new(
    0x000006f2dec549b9438ca9aadd557d69,
    0x9ee191f71a30122e4d1011d1f96a27bc,
    0x7529e3aa1277d0a0179f94911aac9632,
    0x3250a8c671decfe9c6e5e37d15c69646,
);

fn ln(x: &f256) -> FP492 {
    debug_assert!(!x.is_special() && x.is_sign_positive());
    let (mut m, sh) = norm_signif(&x.bits);
    let e = exp(&x.bits) - sh as i32;
    // x = m⋅2⁻ⁿ⋅2ᵉ with n = 236 and 1 < m⋅2⁻ⁿ < 2
    // m has 19 leading zeroes. By using it as the hi-part of an FP492 value,
    // we turn m⋅2⁻ⁿ into m'⋅2⁻ⁿ⁻²⁵⁶, so that
    // x = m'⋅2⁻ⁿ⁻²⁵⁶⋅2ᵉ
    // ln x = ln (m'⋅2⁻ⁿ⁻²⁵⁶) + ln 2ᵉ = ln (m'⋅2⁻ⁿ⁻²⁵⁶) + e⋅ln 2
    let m = FP492::new(m.hi.0, m.lo.0, 0_u128, 0_u128);
    let ln_m = feynman::feynman(&m);
    let mut ln = LN_2;
    ln *= &FP492::from(e);
    ln += &ln_m;
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
        let mut t = ln(&self);
        t *= &LOG2_E;
        Self::from(&t)
    }

    /// Returns the base 10 logarithm of the number.
    pub fn log10(&self) -> Self {
        // x <= 0 or x is infinite or nan => log₁₀ x is nan
        if self.is_special() || self.is_sign_negative() {
            return Self::NAN;
        }
        // log₁₀ x = ln x ⋅ log₁₀ e
        let mut t = ln(&self);
        t *= &LOG10_E;
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
        assert_eq!(x.ln(), -f256::EPSILON - f256::EPSILON.ulp());
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
