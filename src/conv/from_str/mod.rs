// ---------------------------------------------------------------------------
// Copyright:   (c) 2022 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

mod common;
mod fast_approx;
mod fast_exact;
mod float_repr;
mod powers_of_five;
mod slow_exact;

use core::{convert::TryFrom, num::ParseFloatError, str::FromStr};

use fast_exact::fast_exact;
use float_repr::FloatRepr;
use slow_exact::f256_exact;

use super::big_decimal::{Decimal, MAX_DIGITS};
use crate::{f256, u256, HI_FRACTION_BIAS, MIN_GT_ZERO_10_EXP};

/// Minimum possible subnormal power of 10 exponent - adjustment of
/// significand: ⌊(Eₘᵢₙ + 1 - p) × log₁₀(2)⌋ - ⌈p × log₁₀(2)⌉.
pub(self) const MIN_10_EXP_CUTOFF: i32 = -79056;

// The internals of ParseFloatError are not public. The following hack is used
// to return the same errors as f64.
fn err(empty: bool) -> ParseFloatError {
    if empty {
        f64::from_str("").unwrap_err()
    } else {
        f64::from_str("_").unwrap_err()
    }
}

#[inline]
fn calc_normal_f256(
    lit: &str,
    sign: u32,
    exp10: i32,
    signif10: u256,
    signif_truncated: bool,
) -> f256 {
    // The transformation of the decimal representation is implemented as a
    // sequence of faster to slower algorithms, chained together by tail
    // calls.
    fast_exact(lit, sign, exp10, signif10, signif_truncated)
}

impl FromStr for f256 {
    type Err = ParseFloatError;

    fn from_str(lit: &str) -> Result<Self, Self::Err> {
        match FloatRepr::from_str(lit) {
            FloatRepr::EMPTY => Err(err(true)),
            FloatRepr::INVALID => Err(err(false)),
            FloatRepr::NAN => Ok(Self::NAN),
            FloatRepr::INF(sign) => {
                Ok([Self::INFINITY, Self::NEG_INFINITY][sign as usize])
            }
            FloatRepr::NUMBER(repr) => {
                let sign = repr.sign;
                let exp10 = repr.exponent;
                let signif10 = repr.significand;
                // We have a number f with a canonical representation
                // (-1)ˢ × w × 10ᵏ, where s ∈ {0, 1}, |k| < 2³¹, w >= 0 and
                // w < 2²⁵⁶ only if it has not been truncated, i.e. if
                // repr.digit_limit_exceeded is false.
                // We need to transform f(s, w, k) it into one of
                //  • f'(s, m, e) so that f' = (-1)ˢ × (1 + m × 2¹⁻ᵖ) × 2ᵉ and
                //    f' ≈ f, where p = 237, Eₘᵢₙ <= e <= Eₘₐₓ and 0 < m <
                // 2ᵖ⁻¹,  • f'(s, m, e) so that f' = (-1)ˢ ×
                // (m × 2¹⁻ᵖ) × 2⁻²⁶²¹⁴³    where p = 237, e <
                // Eₘᵢₙ and 0 < m < 2ᵖ⁻¹,  • ±0 if w = 0 or e
                // < Eₘᵢₙ + 1 - p,  • ±Infinity if e > Eₘₐₓ.

                // k < ⌊(Eₘᵢₙ + 1 - p) × log₁₀(2)⌋ - ⌈p × log₁₀(2)⌉
                // => e < Eₘᵢₙ + 1 - p
                if signif10.is_zero() || exp10 < MIN_10_EXP_CUTOFF {
                    return Ok([Self::ZERO, Self::NEG_ZERO][sign as usize]);
                }
                // k > ⌊(Eₘᵢₙ + 1 - p) × log₁₀(2)⌋ - ⌈p × log₁₀(2)⌉ and
                // k < ⌊(Eₘᵢₙ + 1 - p) × log₁₀(2)⌋
                // => e < Eₘᵢₙ
                if exp10 < MIN_GT_ZERO_10_EXP {
                    // Subnormals are not handled by the fast algorithms.
                    return Ok(f256_exact(lit));
                }
                // k > ⌊(Eₘₐₓ + 1) × log₁₀(2)⌋ => e > Eₘₐₓ
                if exp10 > Self::MAX_10_EXP {
                    return Ok(
                        [Self::INFINITY, Self::NEG_INFINITY][sign as usize]
                    );
                }
                Ok(calc_normal_f256(
                    lit,
                    sign,
                    exp10,
                    signif10,
                    repr.signif_truncated,
                ))
            }
        }
    }
}

impl TryFrom<&str> for f256 {
    type Error = ParseFloatError;

    #[inline]
    fn try_from(lit: &str) -> Result<Self, Self::Error> {
        Self::from_str(lit)
    }
}

mod cmp_algos_tests {
    use super::{fast_approx::fast_approx, *};

    fn cmp_algos(lit: &str) -> bool {
        if let FloatRepr::NUMBER(repr) = FloatRepr::from_str(lit) {
            let sign = repr.sign;
            let exp10 = repr.exponent;
            let signif10 = repr.significand;
            let signif_truncated = repr.signif_truncated;
            let fe = fast_exact(lit, sign, exp10, signif10, signif_truncated);
            let fa =
                fast_approx(lit, sign, exp10, signif10, signif_truncated);
            let fs = f256_exact(lit);
            if fe != fa || fa != fs {
                println!("> {}", lit);
                println!("> {:?}", fe.decode());
                println!("> {:?}", fa.decode());
                println!("> {:?}", fs.decode());
                return false;
            }
        }
        true
    }

    #[test]
    fn test_1() {
        let lit = "+4970589695.\
                   02834591566739131418985477099711133801877365304479497e-54";
        assert!(cmp_algos(lit));
    }

    #[test]
    fn test_2() {
        let lit = ".923153707130498519861416615062846647251730131089915766587063647843239307e100";
        assert!(cmp_algos(lit));
    }

    #[test]
    fn test_3() {
        let lit = "-097260229193297382461635949130642263176220.0e-63";
        assert!(cmp_algos(lit));
    }

    #[test]
    fn test_4() {
        let lit = "-258163989229583650361874280907281656079733634034956654.\
                   053563825162895329e18";
        assert!(cmp_algos(lit));
    }
}

#[cfg(test)]
mod from_str_tests {
    use super::*;

    #[test]
    fn test_nan() {
        assert!(f256::from_str("nan").unwrap().is_nan());
        assert!(f256::from_str("-nan").unwrap().is_nan());
        assert!(f256::from_str("+nan").unwrap().is_nan());
        assert!(f256::from_str("Nan").unwrap().is_nan());
        assert!(f256::from_str("-Nan").unwrap().is_nan());
        assert!(f256::from_str("+Nan").unwrap().is_nan());
        assert!(f256::from_str("NaN").unwrap().is_nan());
        assert!(f256::from_str("-NaN").unwrap().is_nan());
        assert!(f256::from_str("+NaN").unwrap().is_nan());
    }

    #[test]
    fn test_inf() {
        assert_eq!(f256::from_str("inf").unwrap(), f256::INFINITY);
        assert_eq!(f256::from_str("+inf").unwrap(), f256::INFINITY);
        assert_eq!(f256::from_str("-inf").unwrap(), f256::NEG_INFINITY);
        assert_eq!(f256::from_str("infinity").unwrap(), f256::INFINITY);
        assert_eq!(f256::from_str("+infinity").unwrap(), f256::INFINITY);
        assert_eq!(f256::from_str("-infinity").unwrap(), f256::NEG_INFINITY);
        assert_eq!(f256::from_str("Inf").unwrap(), f256::INFINITY);
        assert_eq!(f256::from_str("+Inf").unwrap(), f256::INFINITY);
        assert_eq!(f256::from_str("-Inf").unwrap(), f256::NEG_INFINITY);
        assert_eq!(f256::from_str("Infinity").unwrap(), f256::INFINITY);
        assert_eq!(f256::from_str("+Infinity").unwrap(), f256::INFINITY);
        assert_eq!(f256::from_str("-Infinity").unwrap(), f256::NEG_INFINITY);
    }

    #[test]
    fn test_zero() {
        assert_eq!(f256::from_str("0").unwrap(), f256::ZERO);
        assert_eq!(f256::from_str("+0").unwrap(), f256::ZERO);
        assert_eq!(f256::from_str("-0").unwrap(), f256::NEG_ZERO);
        assert_eq!(f256::from_str("0000000000").unwrap(), f256::ZERO);
        assert_eq!(f256::from_str("+0000000000").unwrap(), f256::ZERO);
        assert_eq!(f256::from_str("-0000000000").unwrap(), f256::NEG_ZERO);
        assert_eq!(f256::from_str("00.00000000").unwrap(), f256::ZERO);
        assert_eq!(f256::from_str("+0000.000000").unwrap(), f256::ZERO);
        assert_eq!(f256::from_str("-000000000.0").unwrap(), f256::NEG_ZERO);
        assert_eq!(f256::from_str("+0.0e69").unwrap(), f256::ZERO);
    }

    #[test]
    fn test_exp_overflow() {
        let f = f256::from_str("12.5E78915").unwrap();
        assert_eq!(f, f256::INFINITY);
        let f = f256::from_str("-12.5E78915").unwrap();
        assert_eq!(f, f256::NEG_INFINITY);
    }

    #[test]
    fn test_exp_underflow() {
        let f = f256::from_str("10.5E-78985").unwrap();
        assert_eq!(f, f256::ZERO);
        let f = f256::from_str("-0.001e-78981").unwrap();
        assert_eq!(f, f256::ZERO);
    }

    #[test]
    fn test_normal_fast_exact() {
        let f = f256::from_str("17.625").unwrap();
        assert_eq!(f.as_sign_exp_signif(), (0, -3, (0, 141)));
        let s =
            "-7.629394531250000000000000000000000000000000000000000000000000000\
            00000000000000000000000000000000000000000000000000000000000e-06";
        let f = f256::from_str(s).unwrap();
        assert_eq!(f.as_sign_exp_signif(), (1, -17, (0, 1)));
        let s = "1234567890.1234567890123456789012345678901234567890";
        let f = f256::from_str(s).unwrap();
        assert_eq!(
            f.as_sign_exp_signif(),
            (
                0,
                -204,
                (
                    93281312402337715824574088725497,
                    139132118994828259455133236546529242971
                )
            )
        );
        let s = "0.03978e-97";
        let f = f256::from_str(s).unwrap();
        assert_eq!(
            f.as_sign_exp_signif(),
            (
                0,
                -563,
                (
                    352949761382803248364893811984677,
                    334535897686359893047635719463331157031
                )
            )
        );
        let s = "-7.57637797e-77";
        let f = f256::from_str(s).unwrap();
        assert_eq!(
            f.as_sign_exp_signif(),
            (
                1,
                -489,
                (
                    355868925714691431204717007765143,
                    243540868627022737783375260043601712363
                )
            )
        );
    }

    #[test]
    fn test_fast_exact_max_digits_exceeded() {
        let mut s = "1.".to_string();
        s.push_str(&*"9".repeat(70));
        s.push_str(&*"0".repeat(8));
        s.push('2');
        let s = s.as_str();
        let f = f256::from_str(s).unwrap();
        assert_eq!(
            f.as_sign_exp_signif(),
            (
                0,
                -236,
                (
                    649037107316853453566312041152511,
                    340282366920938463463374607431768211445
                )
            )
        );
    }

    #[test]
    fn test_normal_fast_approx() {
        let f = f256::from_str("17.69e107").unwrap();
        assert_eq!(
            f.as_sign_exp_signif(),
            (
                0,
                123,
                (
                    488876229566786321353870606249405,
                    6438732618457514668061927528358337931
                )
            )
        );
        let f = f256::from_str("109.04e-111").unwrap();
        assert_eq!(
            f.as_sign_exp_signif(),
            (
                0,
                -596,
                (
                    83104360821888621315881191879064,
                    58982733340686851635519409389570484615
                )
            )
        );
        let lit = "-258163989229583650361874280907281656079733634034956654.\
                   053563825162895329e18";
        let f = f256::from_str(lit).unwrap();
        assert_eq!(
            f.as_sign_exp_signif(),
            (
                1,
                5,
                (
                    23708618041025113844032871256452,
                    268638390033499559495964339277996026367
                )
            )
        );
    }

    #[test]
    fn test_fast_approx_max_digits_exceeded() {
        let mut s = "1.".to_string();
        s.push_str(&*"9".repeat(70));
        s.push_str(&*"0".repeat(8));
        s.push_str("2e-95");
        let s = s.as_str();
        let f = f256::from_str(s).unwrap();
        assert_eq!(
            f.as_sign_exp_signif(),
            (
                0,
                -550,
                (
                    216614819853188660904563608136178,
                    140989523720527938018691533154554017175
                )
            )
        );
        let s = "-3399920010740781265762287772454133426222426857980018648\
                65254048807018638070632.0e-475";
        let f = f256::from_str(s).unwrap();
        assert_eq!(
            f.as_sign_exp_signif(),
            (
                1,
                -1556,
                (
                    252523911500343292542282777968512,
                    306619672534410476792177550986574773341
                )
            )
        );
    }

    #[test]
    fn test_slow_exact() {
        let s = "+6693707603597347117297158868310984450882752298764236217\
                5927640154509878799559.0e874";
        let f = f256::from_str(s).unwrap();
        assert_eq!(
            f.as_sign_exp_signif(),
            (
                0,
                2926,
                (
                    30203688295486241190210752268742,
                    58986786656108783719011333115234100195
                )
            )
        );
    }

    #[test]
    fn test_subnormal() {
        let s = "0145441.249009748590979791323783709646682894752724672748\
                600542581589e-78928";
        let f = f256::from_str(s).unwrap();
        assert_eq!(
            f.as_sign_exp_signif(),
            (
                0,
                -262377,
                (
                    9506496199230481144411,
                    183478489063339905949211997661958933079
                )
            )
        );
        let s = "-0.9818036132127703363504450836394764653184121e-78913";
        let f = f256::from_str(s).unwrap();
        assert_eq!(
            f.as_sign_exp_signif(),
            (
                1,
                -262378,
                (
                    128347527004149295075436743924545,
                    200698461692417807477600193256349332369
                )
            )
        );
    }

    #[test]
    fn test_subnormal_near_zero() {
        // let s = "1.125e-78984";
        // let f = f256::from_str(s).unwrap();
        // assert_eq!(f, f256::MIN_GT_ZERO);
        // assert_eq!(f.as_sign_exp_signif(), (0, -262378, (0, 1)));
        // let s = "-5.625e-78983";
        // let f = f256::from_str(s).unwrap();
        // assert_eq!(f.as_sign_exp_signif(), (1, -262378, (0, 25)));
        let s = "-021.75e-78985";
        let f = f256::from_str(s).unwrap();
        assert_eq!(f.as_sign_exp_signif(), (1, -262378, (0, 1)));
    }

    #[test]
    fn test_err_empty_str() {
        let res = f256::from_str("");
        assert!(res.is_err());
    }

    #[test]
    fn test_err_invalid_lit() {
        let lits = [" ", "+", "-4.33.2", "2.87 e3", "+e3", ".4e3 "];
        for lit in lits {
            let res = f256::from_str(lit);
            assert!(res.is_err());
        }
    }
}
