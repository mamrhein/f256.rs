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
mod slow_exact;

use core::{convert::TryFrom, num::ParseFloatError, str::FromStr};

use fast_approx::try_fast_approx;
use fast_exact::try_fast_exact;
use float_repr::FloatRepr;
use slow_exact::f256_exact;

use crate::{f256, u256, HI_FRACTION_BIAS, MIN_GT_ZERO_10_EXP};

// The internals of ParseFloatError are not public. The following hack is used
// to return the same errors as f64.
fn err(empty: bool) -> ParseFloatError {
    if empty {
        f64::from_str("").unwrap_err()
    } else {
        f64::from_str("_").unwrap_err()
    }
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
                let s = repr.sign;
                let w = repr.significand;
                let k = repr.exponent;
                // We have a number with a canonical representation
                // (-1)ˢ × w × 10ᵏ, where s ∈ {0, 1}, |k| < 2³¹, w >= 0 and
                // w < 2²⁵⁶ only if it has not been truncated, i.e.
                // repr.digit_limit_exceeded is false.
                // We need to transform it into (-1)ˢ × (1 + m × 2¹⁻ᵖ) × 2ᵉ,
                // where p = 237, Eₘᵢₙ <= e - Eₘₐₓ <= Eₘₐₓ and 0 < m < 2ᵖ⁻¹,
                // or - if w = 0 or k < ⌊(Eₘᵢₙ + 1 - p) × log₁₀(2)⌋ - into ±0,
                // or - if k > ⌊(Eₘₐₓ + 1) × log₁₀(2)⌋ - into ±Infinity.
                if w.is_zero() || k < MIN_GT_ZERO_10_EXP {
                    return Ok([Self::ZERO, Self::NEG_ZERO][s as usize]);
                }
                if k > Self::MAX_10_EXP {
                    return Ok([Self::INFINITY, Self::NEG_INFINITY][s as usize]);
                }
                if let Some(f) = try_fast_exact(s, w, k) {
                    if !repr.digit_limit_exceeded {
                        return Ok(f);
                    } else {
                        // The real significand w' has been truncated, so f may
                        // by less than the correctly rounded result f'. But
                        // w < w' < w+1 => f <= f' <= g where g is the
                        // transformation of (-1)ˢ × (w+1) × 10ᵏ. If f = g
                        // then f = f'.
                        let mut wp1 = w;
                        wp1.incr();
                        if !wp1.is_zero() {
                            // Otherwise wp1 overflowed!
                            if let Some(g) = try_fast_exact(s, wp1, k) {
                                if f == g {
                                    return Ok(f);
                                }
                            }
                        }
                    }
                }
                if let Some(f) = try_fast_approx(s, w, k) {
                    return Ok(f);
                }
                // The last resort must always succed!
                Ok(f256_exact(lit))
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
