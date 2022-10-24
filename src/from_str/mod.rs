// ---------------------------------------------------------------------------
// Copyright:   (c) 2022 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

mod big_num;
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

use crate::{f256, u256, HI_FRACTION_BIAS, MIN_GT_ZERO_10_EXP};

/// Minimum possible subnormal power of 10 exponent - adjustment of significand:
/// ⌊(Eₘᵢₙ + 1 - p) × log₁₀(2)⌋ - ⌈p × log₁₀(2)⌉.
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
    // sequence of faster to slower algorithms, chained together by tail calls.
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
                //    f' ≈ f, where p = 237, Eₘᵢₙ <= e <= Eₘₐₓ and 0 < m < 2ᵖ⁻¹,
                //  • f'(s, m, e) so that f' = (-1)ˢ × (m × 2¹⁻ᵖ) × 2⁻²⁶²¹⁴³
                //    where p = 237, e < Eₘᵢₙ and 0 < m < 2ᵖ⁻¹,
                //  • ±0 if w = 0 or e < Eₘᵢₙ + 1 - p,
                //  • ±Infinity if e > Eₘₐₓ.

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

mod tests {
    use super::{fast_approx::fast_approx, *};

    fn cmp_algos(lit: &str) -> bool {
        if let FloatRepr::NUMBER(repr) = FloatRepr::from_str(lit) {
            let sign = repr.sign;
            let exp10 = repr.exponent;
            let signif10 = repr.significand;
            let signif_truncated = repr.signif_truncated;
            let fe = fast_exact(lit, sign, exp10, signif10, signif_truncated);
            let fa = fast_approx(lit, sign, exp10, signif10, signif_truncated);
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
