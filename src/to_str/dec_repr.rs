// ---------------------------------------------------------------------------
// Copyright:   (c) 2022 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use core::{cmp::min, fmt};
use std::{cmp::max, fmt::Write};

use super::{ge_lut::from_ge_lut, lt_lut::from_lt_lut};
use crate::{
    f256,
    to_str::powers_of_five::is_multiple_of_pow5,
    u256,
    uint256::{u256_truncating_mul, u256_truncating_mul_u512},
};

/// Returns ⌊log₁₀(2ⁱ)⌋ for 0 <= i <= 262144.
#[inline(always)]
fn floor_log10_pow2(i: i32) -> i32 {
    debug_assert!(i >= 0);
    debug_assert!(i <= 262144);
    ((i as u128 * 169464822037455) >> 49) as i32
}

/// Returns ⌊log₁₀(5ⁱ)⌋ for 0 <= i <= 262144.
#[inline(always)]
fn floor_log10_pow5(i: i32) -> i32 {
    debug_assert!(i >= 0);
    debug_assert!(i <= 262144);
    ((i as u128 * 24592820711491) >> 45) as i32
}

/// Returns ⌊log₂(5ⁱ)⌋ for 0 <= i <= 225798.
#[inline(always)]
fn floor_log2_pow5(i: i32) -> i32 {
    debug_assert!(i >= 0);
    debug_assert!(i <= 225798);
    ((i as u64 * 81695582054029) >> 45) as i32
}

/// Returns ⌈log₂(5ⁱ)⌉ for 0 <= i <= 225798.
#[inline(always)]
fn ceil_log2_pow5(i: i32) -> i32 {
    floor_log2_pow5(i) + 1
}

/// Internal representation of a decimal number d as (s, k, w) where s ∈ {0, 1},
/// |k| < 2³¹, 0 <= w < 2²⁵⁶ and d = (-1)ˢ × w × 10ᵏ.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(super) struct DecNumRepr {
    pub(super) sign: u32,
    pub(super) exp10: i32,
    pub(super) signif10: u256,
}

impl DecNumRepr {
    /// Converts a finite, non-zero `f256` value into its shortest, correctly
    /// rounded decimal representation.
    pub(super) fn from_f256_shortest(f: &f256) -> Self {
        // This is an implementation of the algorithm presented by Ulf Adams in
        // his PLDI'18 paper `Ryū: Fast Float-to-String Conversion`, available
        // at [https://dl.acm.org/doi/pdf/10.1145/3296979.3192369], adapted to
        // f256.
        debug_assert!(f.is_finite() && !f.is_zero());

        // Max number of bits needed to store ⌊2ʰ / 5ᵍ⌋ + 1 or ⌊5⁻ᵉ⁻ᵍ / 2ʰ⌋.
        const H: i32 = 501;

        // Step 1: Decode the binary floating-point number.
        let sign = f.sign();
        // Subtract 2 from exponent and adjust significand in prep of step 2.
        let exp2 = f.exponent() - 2;
        let signif2 = f.significand() << 2;
        let accept_bounds = signif2.lo.is_power_of_two();

        // Step 2: Compute the halfway points to the next smaller and larger
        // floating point values.
        let is_non_integer = !f.fraction().is_zero() || exp2 <= -1;
        let lower_signif2 = signif2 - 1 - is_non_integer as u32;
        let upper_signif2 = signif2 + 2;

        // Step 3: Convert the interval to a decimal power base.
        let mut exp10: i32;
        let mut signif10 = u256::default();
        let mut lower_signif10 = u256::default();
        let mut upper_signif10 = u256::default();
        let mut rem_zero = false;
        let mut lower_rem_zero = false;
        if exp2 >= 0 {
            // g = max(0, ⌊e₂ × log₁₀(2)⌋ - 1)
            // ⌊e₂ × log₁₀(2)⌋ = 0 for e₂ ∈ [0..3], therefor the following
            // expression is equivalent to the one above.
            let g = floor_log10_pow2(exp2) - (exp2 > 3) as i32;
            exp10 = g;
            let h = floor_log2_pow5(g) + H;
            let d = -exp2 + g + h - 512;
            let shr = max(d, 0) as u32;
            let shl = max(-d, 0) as u32;
            let luv = from_ge_lut(g as usize);
            lower_signif10 =
                u256_truncating_mul_u512(&(lower_signif2 << shl), &luv) >> shr;
            signif10 = u256_truncating_mul_u512(&(signif2 << shl), &luv) >> shr;
            upper_signif10 =
                u256_truncating_mul_u512(&(upper_signif2 << shl), &luv) >> shr;
            // exp2 >= 0 => rem_zero = signif10 % 10ᵍ == 0 = signif2 % 5ᵍ == 0
            // Analog for the lower and upper bound.
            // exp2 >= 0 => g >= 0
            // signif2 and its lower and upper bounds have atmost 239 bits.
            // g > 102 => 5ᵍ > 2²³⁹ => signif2 and its bounds can't be a
            // multiple of 5ᵍ.
            if g <= 102 {
                // Only one of lower_signif10, signif10, upper_signif10 can be a
                // multiple of 5, if any.
                if signif2 % 5_u32 == 0 {
                    rem_zero = is_multiple_of_pow5(signif2, g as u32);
                } else if accept_bounds {
                    lower_rem_zero =
                        is_multiple_of_pow5(lower_signif2, g as u32);
                } else if is_multiple_of_pow5(upper_signif2, g as u32) {
                    upper_signif10.decr();
                }
            }
        } else {
            // e₂ < 0
            // g = max(0, ⌊-e₂ × log₁₀(5)⌋ - 1)
            // ⌊-e₂ × log₁₀(5)⌋ = 0 for e₂ = -1, therefor the following
            // expression is equivalent to the one above.
            let g = floor_log10_pow5(-exp2) - (exp2 != -1) as i32;
            exp10 = exp2 + g;
            let i = -exp2 - g;
            let h = ceil_log2_pow5(i) - H;
            let d = g - h - 512;
            let shr = max(d, 0) as u32;
            let shl = max(-d, 0) as u32;
            let luv = from_lt_lut(i as usize);
            lower_signif10 =
                u256_truncating_mul_u512(&(lower_signif2 << shl), &luv) >> shr;
            signif10 = u256_truncating_mul_u512(&(signif2 << shl), &luv) >> shr;
            upper_signif10 =
                u256_truncating_mul_u512(&(upper_signif2 << shl), &luv) >> shr;
            // exp2 < 0 => rem_zero = signif10 % 10ᵍ == 0 = signif2 % 2ᵍ == 0
            // Analog for the lower and upper bound.
            if g <= 1 {
                // signif2 = 4 * f.significand, so it has atleast 2 trailing
                // zero bits.
                rem_zero = true;
                if accept_bounds {
                    // lower_signif2 = signif2 - 1 - is_non_integer, so it has
                    // a trailing zero bit if f is not an integer.
                    lower_rem_zero = is_non_integer;
                } else {
                    // uppper_signif2 = signif2 + 2, so it always has at least
                    // one trailing zero bit.
                    signif10.decr();
                }
            } else if g <= 238 {
                // signif2 has atmost 239 bits, i.e atmost 238 trailing zeroes.
                rem_zero = signif2.trailing_zeros() >= g as u32;
            }
        }

        // Step 4: Find the shortest, correctly-rounded representation within
        // this interval.
        println!("{f:?}");
        println!("{lower_rem_zero} {rem_zero}");
        let mut i = 0_i32;
        let mut round_digit = 0_u64;
        if lower_rem_zero || rem_zero {
            let (mut lower_quot, mut lower_rem) =
                lower_signif10.divmod_pow10(1);
            let (mut upper_quot, mut upper_rem) =
                upper_signif10.divmod_pow10(1);
            while lower_quot < upper_quot {
                (signif10, round_digit) = signif10.divmod_pow10(1);
                rem_zero &= round_digit == 0;
                lower_signif10 = lower_quot;
                lower_rem_zero &= lower_rem == 0;
                upper_signif10 = upper_quot;
                i += 1;
                (lower_quot, lower_rem) = lower_signif10.divmod_pow10(1);
                (upper_quot, upper_rem) = upper_signif10.divmod_pow10(1);
            }
            if lower_rem_zero {
                while lower_rem == 0 && !lower_signif10.is_zero() {
                    (signif10, round_digit) = signif10.divmod_pow10(1);
                    rem_zero &= round_digit == 0;
                    lower_signif10 = lower_quot;
                    (lower_quot, lower_rem) = lower_signif10.divmod_pow10(1);
                    i += 1;
                }
            }
            if round_digit > 5  // need to round up
                || (round_digit == 5    // need to round to even
                && (!rem_zero || (rem_zero && (signif10.lo & 1) == 1)))
                || signif10 == lower_signif10
            // signif10 out of bounds
            {
                signif10.incr();
            }
        } else {
            // Can't have a tie.
            let mut round_up = false;
            // First, try to remove two digits.
            let (mut lower_quot, mut lower_rem) =
                lower_signif10.divmod_pow10(2);
            let (mut upper_quot, mut upper_rem) =
                upper_signif10.divmod_pow10(2);
            if upper_quot > lower_quot {
                let (quot, rem) = signif10.divmod_pow10(2);
                round_up = rem >= 50;
                signif10 = quot;
                lower_signif10 = lower_quot;
                upper_signif10 = upper_quot;
                i += 2;
            }
            (lower_quot, lower_rem) = lower_signif10.divmod_pow10(1);
            (upper_quot, upper_rem) = upper_signif10.divmod_pow10(1);
            while upper_quot > lower_quot {
                (signif10, round_digit) = signif10.divmod_pow10(1);
                round_up = round_digit >= 5;
                lower_signif10 = lower_quot;
                upper_signif10 = upper_quot;
                (lower_quot, lower_rem) = lower_signif10.divmod_pow10(1);
                (upper_quot, upper_rem) = upper_signif10.divmod_pow10(1);
                i += 1;
            }
            if round_up || signif10 == lower_signif10 {
                signif10.incr();
            }
        }
        // Adjust exponent by adding number of removed digits
        exp10 += i;

        Self {
            sign,
            exp10,
            signif10,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_one() {
        let f = f256::ONE;
        let r = DecNumRepr::from_f256_shortest(&f);
        assert_eq!(
            r,
            DecNumRepr {
                sign: 0,
                exp10: 0,
                signif10: u256::new(0, 1)
            }
        )
    }

    #[test]
    fn test_ten_pow_72() {
        let f = f256::from_str("10.0e72").unwrap();
        let r = DecNumRepr::from_f256_shortest(&f);
        assert_eq!(
            r,
            DecNumRepr {
                sign: 0,
                exp10: 73,
                signif10: u256::new(0, 1)
            }
        )
    }

    #[test]
    fn test_one_half() {
        let f = f256::encode(0, -1, u256::new(0, 1));
        let r = DecNumRepr::from_f256_shortest(&f);
        assert_eq!(
            r,
            DecNumRepr {
                sign: 0,
                exp10: -1,
                signif10: u256::new(0, 5)
            }
        )
    }

    #[test]
    fn test_one_sixteenth() {
        let f = f256::encode(0, -4, u256::new(0, 1));
        let r = DecNumRepr::from_f256_shortest(&f);
        assert_eq!(
            r,
            DecNumRepr {
                sign: 0,
                exp10: -4,
                signif10: u256::new(0, 625)
            }
        )
    }

    #[test]
    fn test_f256_max() {
        let f = f256::MAX;
        let r = DecNumRepr::from_f256_shortest(&f);
        assert_eq!(
            r,
            DecNumRepr {
                sign: 0,
                exp10: 78842,
                signif10: u256::new(
                    473526069559795162737608364600986,
                    168794288209602616731974382256735511567
                )
            }
        )
    }
}
