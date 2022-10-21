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

use crate::{
    f256, from_str::slow_exact::f256_exact, u256, HI_FRACTION_BIAS,
    MIN_GT_ZERO_10_EXP,
};

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
