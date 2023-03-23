// ---------------------------------------------------------------------------
// Copyright:   (c) 2021 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

#![doc = include_str ! ("../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]
// activate some rustc lints
#![deny(non_ascii_idents)]
#![deny(unsafe_code)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]
#![warn(trivial_casts, trivial_numeric_casts)]
// TODO: switch to #![warn(unused)]
#![allow(unused)]
#![allow(dead_code)]
// activate some clippy lints
#![warn(clippy::cast_possible_truncation)]
#![warn(clippy::cast_possible_wrap)]
#![warn(clippy::cast_precision_loss)]
#![warn(clippy::cast_sign_loss)]
#![warn(clippy::cognitive_complexity)]
#![warn(clippy::decimal_literal_representation)]
#![warn(clippy::enum_glob_use)]
#![warn(clippy::equatable_if_let)]
#![warn(clippy::fallible_impl_from)]
#![warn(clippy::if_not_else)]
#![warn(clippy::if_then_some_else_none)]
#![warn(clippy::implicit_clone)]
#![warn(clippy::integer_division)]
#![warn(clippy::manual_assert)]
#![warn(clippy::match_same_arms)]
#![warn(clippy::mismatching_type_param_order)]
#![warn(clippy::missing_const_for_fn)]
#![warn(clippy::missing_errors_doc)]
#![warn(clippy::missing_panics_doc)]
#![warn(clippy::multiple_crate_versions)]
#![warn(clippy::multiple_inherent_impl)]
#![warn(clippy::must_use_candidate)]
#![warn(clippy::needless_pass_by_value)]
#![warn(clippy::print_stderr)]
#![warn(clippy::print_stdout)]
#![warn(clippy::semicolon_if_nothing_returned)]
#![warn(clippy::str_to_string)]
#![warn(clippy::string_to_string)]
#![warn(clippy::undocumented_unsafe_blocks)]
#![warn(clippy::unicode_not_nfc)]
#![warn(clippy::unimplemented)]
#![warn(clippy::unseparated_literal_suffix)]
#![warn(clippy::unused_self)]
#![warn(clippy::unwrap_in_result)]
#![warn(clippy::use_self)]
#![warn(clippy::used_underscore_binding)]
#![warn(clippy::wildcard_imports)]

extern crate core;

use core::{
    cmp::{max, Ordering},
    num::FpCategory,
    ops::Neg,
};

mod big_uint;
mod binops;
pub mod consts;
mod conv;
mod math;

use crate::big_uint::{u256, u512};

/// Precision level in relation to single precision float (f32) = 8
pub(crate) const PREC_LEVEL: u32 = 8;
/// Total number of bits = 256
pub(crate) const TOTAL_BITS: u32 = 1_u32 << PREC_LEVEL;
/// Number of exponent bits = 19
pub(crate) const EXP_BITS: u32 = 4 * PREC_LEVEL - 13;
/// Number of significand bits p = 237
pub(crate) const SIGNIFICAND_BITS: u32 = TOTAL_BITS - EXP_BITS;
/// Number of fraction bits = p - 1 = 236
pub(crate) const FRACTION_BITS: u32 = SIGNIFICAND_BITS - 1;
/// Maximum value of biased base 2 exponent = 0x7ffff = 524287
pub(crate) const EXP_MAX: u32 = (1_u32 << EXP_BITS) - 1;
/// Base 2 exponent bias = 0x3ffff = 262143
pub(crate) const EXP_BIAS: u32 = EXP_MAX >> 1;
/// Maximum value of base 2 exponent = 0x3ffff = 262143
#[allow(clippy::cast_possible_wrap)]
pub(crate) const EMAX: i32 = (EXP_MAX >> 1) as i32;
/// Minimum value of base 2 exponent = -262142
pub(crate) const EMIN: i32 = 1 - EMAX;
/// Number of bits in hi u128
pub(crate) const HI_TOTAL_BITS: u32 = TOTAL_BITS >> 1;
/// Number of bits to shift right for sign = 127
pub(crate) const HI_SIGN_SHIFT: u32 = HI_TOTAL_BITS - 1;
/// Number of fraction bits in hi u128 = 108
pub(crate) const HI_FRACTION_BITS: u32 = FRACTION_BITS - HI_TOTAL_BITS;
/// Fraction bias in hi u128 = 2¹⁰⁸ = 0x1000000000000000000000000000
pub(crate) const HI_FRACTION_BIAS: u128 = 1_u128 << HI_FRACTION_BITS;
/// Fraction mask in hi u128 = 0xfffffffffffffffffffffffffff
pub(crate) const HI_FRACTION_MASK: u128 = HI_FRACTION_BIAS - 1;
/// Exponent mask in hi u128 = 0x7ffff000000000000000000000000000
pub(crate) const HI_EXP_MASK: u128 = (EXP_MAX as u128) << HI_FRACTION_BITS;
/// Sign mask in hi u128 = 0x80000000000000000000000000000000
pub(crate) const HI_SIGN_MASK: u128 = 1_u128 << HI_SIGN_SHIFT;
/// Abs mask in hi u128 = 0x7fffffffffffffffffffffffffffffff
pub(crate) const HI_ABS_MASK: u128 = !HI_SIGN_MASK;
/// Value of hi u128 for NaN = 0x7ffff800000000000000000000000000
pub(crate) const NAN_HI: u128 =
    HI_EXP_MASK | (1_u128 << (HI_FRACTION_BITS - 1));
/// Value of hi u128 for Inf = 0x7ffff000000000000000000000000000
pub(crate) const INF_HI: u128 = HI_EXP_MASK;
/// Value of hi u128 for -Inf = 0xfffff000000000000000000000000000
pub(crate) const NEG_INF_HI: u128 = HI_SIGN_MASK | HI_EXP_MASK;
/// Value of hi u128 for epsilon = 0x3ff13000000000000000000000000000
pub(crate) const EPSILON_HI: u128 =
    ((EXP_BIAS - FRACTION_BITS) as u128) << HI_FRACTION_BITS;
/// Value of hi u128 for MAX = 0x7fffefffffffffffffffffffffffffff
pub(crate) const MAX_HI: u128 =
    ((EMAX as u32 + EXP_BIAS) as u128) << HI_FRACTION_BITS | HI_FRACTION_MASK;
/// Binary exponent for integral values
#[allow(clippy::cast_possible_wrap)]
pub(crate) const INT_EXP: i32 = -(FRACTION_BITS as i32);
/// Value of hi u128 of the smallest f256 value with no fractional part (2²³⁶)
pub(crate) const MIN_NO_FRACT_HI: u128 =
    ((EXP_BIAS + FRACTION_BITS) as u128) << HI_FRACTION_BITS;
/// Minimum possible subnormal power of 10 exponent =
/// ⌊(Eₘᵢₙ + 1 - p) × log₁₀(2)⌋.
pub(crate) const MIN_GT_ZERO_10_EXP: i32 = -78984;

/// A 256-bit floating point type (specifically, the “binary256” type defined in
/// IEEE 754-2008).
///
/// For details see [above](index.html).
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Default)]
pub struct f256 {
    pub(crate) bits: u256,
}

/// Some f256 constants (only used to hide the internals in the doc)
const EPSILON: f256 = f256 {
    bits: u256 {
        hi: EPSILON_HI,
        lo: 0,
    },
};
const MAX: f256 = f256 {
    bits: u256 {
        hi: MAX_HI,
        lo: u128::MAX,
    },
};
const MIN: f256 = MAX.negated();
const MIN_POSITIVE: f256 = f256 {
    bits: u256 {
        hi: HI_FRACTION_BIAS,
        lo: 0,
    },
};
const MIN_GT_ZERO: f256 = f256 {
    bits: u256 { hi: 0, lo: 1 },
};
const NAN: f256 = f256 {
    bits: u256 { hi: NAN_HI, lo: 0 },
};
const INFINITY: f256 = f256 {
    bits: u256 { hi: INF_HI, lo: 0 },
};
const NEG_INFINITY: f256 = f256 {
    bits: u256 {
        hi: NEG_INF_HI,
        lo: 0,
    },
};
const ZERO: f256 = f256 {
    bits: u256 { hi: 0, lo: 0 },
};
const NEG_ZERO: f256 = ZERO.negated();
pub(crate) const ONE_HALF: f256 = f256 {
    bits: u256 {
        hi: ((EXP_BIAS - 1) as u128) << HI_FRACTION_BITS,
        lo: 0,
    },
};
const ONE: f256 = f256 {
    bits: u256 {
        hi: (EXP_BIAS as u128) << HI_FRACTION_BITS,
        lo: 0,
    },
};
const NEG_ONE: f256 = ONE.negated();
const TWO: f256 = f256 {
    bits: u256 {
        hi: ((1 + EXP_BIAS) as u128) << HI_FRACTION_BITS,
        lo: 0,
    },
};
pub(crate) const FIVE: f256 = f256::from_u64(5);
const TEN: f256 = f256::from_u64(10);

#[allow(clippy::multiple_inherent_impl)]
impl f256 {
    /// The radix or base of the internal representation of `f256`.
    pub const RADIX: u32 = 2;

    /// Number of significant digits in base 2: 237.
    pub const MANTISSA_DIGITS: u32 = SIGNIFICAND_BITS;

    /// Number of significant digits in base 2: 237.
    pub const SIGNIFICANT_DIGITS: u32 = SIGNIFICAND_BITS;

    /// Approximate number of significant digits in base 10: ⌊log₁₀(2²³⁷)⌋.
    pub const DIGITS: u32 = 71;

    /// The difference between `1.0` and the next larger representable number:
    /// 2⁻²³⁶ ≈ 9.055679 × 10⁻⁷².
    pub const EPSILON: Self = EPSILON;

    /// Largest finite `f256` value:  2²⁶²¹⁴⁴ − 2²⁶¹⁹⁰⁷ ≈ 1.6113 × 10⁷⁸⁹¹³.
    pub const MAX: Self = MAX;

    /// Smallest finite `f256` value: 2²⁶¹⁹⁰⁷ - 2²⁶²¹⁴⁴ ≈ -1.6113 × 10⁷⁸⁹¹³.
    pub const MIN: Self = MIN;

    /// Smallest positive normal `f256` value: 2⁻²⁶²¹⁴² ≈ 2.4824 × 10⁻⁷⁸⁹¹³.
    pub const MIN_POSITIVE: Self = MIN_POSITIVE;

    /// Smallest positive subnormal `f256` value: 2⁻²⁶²³⁷⁸ ≈ 2.248 × 10⁻⁷⁸⁹⁸⁴.
    pub const MIN_GT_ZERO: Self = MIN_GT_ZERO;

    /// Maximum possible power of 2 exponent: Eₘₐₓ + 1 = 2¹⁸ = 262144.
    pub const MAX_EXP: i32 = EMAX + 1;

    /// One greater than the minimum possible normal power of 2 exponent:
    /// Eₘᵢₙ + 1 = -262141.
    pub const MIN_EXP: i32 = EMIN + 1;

    /// Maximum possible power of 10 exponent: ⌊(Eₘₐₓ + 1) × log₁₀(2)⌋.
    pub const MAX_10_EXP: i32 = 78913;

    /// Minimum possible normal power of 10 exponent:
    /// ⌊(Eₘᵢₙ + 1) × log₁₀(2)⌋.
    pub const MIN_10_EXP: i32 = -78912;

    /// Not a Number (NaN).
    ///
    /// Note that IEEE-745 doesn't define just a single NaN value; a plethora of
    /// bit patterns are considered to be NaN. Furthermore, the standard makes a
    /// difference between a "signaling" and a "quiet" NaN, and allows
    /// inspecting its "payload" (the unspecified bits in the bit pattern).
    /// This implementation does not make such a difference and uses exactly one
    /// bit pattern for NaN.
    pub const NAN: Self = NAN;

    /// Infinity (∞).
    pub const INFINITY: Self = INFINITY;

    /// Negative infinity (−∞).
    pub const NEG_INFINITY: Self = NEG_INFINITY;

    /// Additive identity (0.0).
    pub const ZERO: Self = ZERO;

    /// Negative additive identity (-0.0).
    pub const NEG_ZERO: Self = NEG_ZERO;

    /// Multiplicative identity (1.0).
    pub const ONE: Self = ONE;

    /// Multiplicative negator (-1.0).
    pub const NEG_ONE: Self = NEG_ONE;

    /// Equivalent of binary base (2.0).
    pub const TWO: Self = TWO;

    /// Equivalent of decimal base (10.0).
    pub const TEN: Self = TEN;

    /// Raw assembly from significand, biased exponent and sign.
    #[inline]
    pub(crate) const fn new(
        significand: u256,
        biased_exponent: u32,
        sign: u32,
    ) -> Self {
        Self {
            bits: u256 {
                hi: (significand.hi & HI_FRACTION_MASK)
                    | ((biased_exponent as u128) << HI_FRACTION_BITS)
                    | ((sign as u128) << HI_SIGN_SHIFT),
                lo: significand.lo,
            },
        }
    }

    /// Construct a finite, non-zero `f256` value f from sign s, exponent t
    /// and significand c,
    ///
    /// where
    ///
    /// * p = 237
    /// * Eₘₐₓ = 262143
    /// * Eₘᵢₙ = 1 - Eₘₐₓ = -262142
    /// * s ∈ {0, 1}
    /// * Eₘᵢₙ - p + 1 <= t <= Eₘₐₓ - p + 1
    /// * 0 <= c < 2ᵖ
    ///
    /// so that f = (-1)ˢ × 2ᵗ × c.
    #[allow(clippy::cast_possible_wrap)]
    #[allow(clippy::cast_sign_loss)]
    pub(crate) fn encode(s: u32, mut t: i32, mut c: u256) -> Self {
        debug_assert!(s == 0 || s == 1);
        debug_assert!(
            t >= EMIN - FRACTION_BITS as i32
                && t <= EMAX - FRACTION_BITS as i32
        );
        debug_assert!(!c.is_zero());
        // We have an integer based representation `(-1)ˢ × 2ᵗ × c` and need to
        // transform it into a fraction based representation
        // `(-1)ˢ × 2ᵉ × (1 + m × 2¹⁻ᵖ)`,
        // where `Eₘᵢₙ <= e <= Eₘₐₓ` and `0 < m < 2ᵖ⁻¹`, or
        // `(-1)ˢ × 2ᵉ × m × 2¹⁻ᵖ`,
        // where `e = Eₘᵢₙ - 1` and `0 < m < 2ᵖ⁻¹`.

        // 1. Compensate radix shift
        t += FRACTION_BITS as i32;
        // 2. Normalize significand
        let nlz = c.leading_zeros();
        // The position of the most significant bit is `256 - nlz - 1`. We need
        // to shift it to the position of the hidden bit, which is
        // `256 - EXP_BITS - 1`. So we have to shift by |nlz - EXP_BITS|.
        match nlz.cmp(&EXP_BITS) {
            Ordering::Greater => {
                // Shift left.
                let shift = (nlz - EXP_BITS);
                if t >= EMIN + shift as i32 {
                    c <<= shift;
                    t -= shift as i32;
                } else {
                    // Number is subnormal
                    c <<= (t - EMIN) as u32;
                    t = EMIN - 1;
                }
            }
            Ordering::Less => {
                // Shift right and round.
                let shift = (EXP_BITS - nlz);
                t += shift as i32;
                c.idiv_pow2(shift);
                // Rounding may have caused significand to overflow.
                if (c.hi >> (HI_FRACTION_BITS + 1)) != 0 {
                    t += 1;
                    c >>= 1;
                }
            }
            _ => {}
        }
        // 3. Offset exponent
        t += EXP_BIAS as i32;
        debug_assert!(t >= 0);
        // 4. Assemble struct f256
        Self::new(c, t as u32, s)
    }

    /// Only public for testing!!!
    #[doc(hidden)]
    #[must_use]
    pub fn from_sign_exp_signif(s: u32, t: i32, c: (u128, u128)) -> Self {
        Self::encode(s, t, u256::new(c.0, c.1))
    }

    /// Returns the sign bit of `self`: 0 = positive, 1 = negative.
    #[inline]
    pub(crate) const fn sign(&self) -> u32 {
        (self.bits.hi >> HI_SIGN_SHIFT) as u32
    }

    /// Returns the biased binary exponent of `self`.
    #[inline]
    pub(crate) const fn biased_exponent(&self) -> u32 {
        ((self.bits.hi & HI_EXP_MASK) >> HI_FRACTION_BITS) as u32
    }

    /// Returns the binary exponent of `self`.
    #[inline]
    #[allow(clippy::cast_possible_wrap)]
    pub(crate) const fn exponent(&self) -> i32 {
        debug_assert!(
            self.is_finite(),
            "Attempt to extract exponent from Infinity or NaN."
        );
        const TOTAL_BIAS: i32 = EXP_BIAS as i32 + FRACTION_BITS as i32;
        let mut exp = self.biased_exponent() as i32;
        exp += (exp == 0) as i32; // Adjust exp for subnormals.
        (!self.is_zero() as i32) * (exp - TOTAL_BIAS)
    }

    /// Returns the fraction of `self`.
    #[inline]
    pub(crate) const fn fraction(&self) -> u256 {
        u256 {
            hi: self.bits.hi & HI_FRACTION_MASK,
            lo: self.bits.lo,
        }
    }

    /// Returns the significand of `self`.
    /// Pre-condition: `self` is finite!
    #[inline]
    pub(crate) const fn significand(&self) -> u256 {
        debug_assert!(
            self.is_finite(),
            "Attempt to extract significand from Infinity or NaN."
        );
        let hidden_one =
            ((self.biased_exponent() != 0) as u128) << HI_FRACTION_BITS;
        u256 {
            hi: (self.bits.hi & HI_FRACTION_MASK) | hidden_one,
            lo: self.bits.lo,
        }
    }

    /// Extract sign, exponent and significand from self
    pub(crate) fn split_raw(&self) -> (u32, i32, u256) {
        const TOTAL_BIAS: i32 = EXP_BIAS as i32 + FRACTION_BITS as i32;
        let sign = self.sign();
        let biased_exp = self.biased_exponent();
        let fraction = self.fraction();
        match (biased_exp, fraction) {
            (0, u256::ZERO) => (sign, 0, u256::ZERO),
            (0, _) => (sign, EMIN, fraction),
            (EXP_MAX, _) => (sign, biased_exp as i32, fraction),
            _ => (
                sign,
                biased_exp as i32 - TOTAL_BIAS,
                u256 {
                    hi: fraction.hi | HI_FRACTION_BIAS,
                    lo: fraction.lo,
                },
            ),
        }
    }

    /// Extract sign s, exponent t and significand c from a finite, non-zero
    /// `f256` f,
    ///
    /// where
    ///
    /// * p = 237
    /// * Eₘₐₓ = 262143
    /// * Eₘᵢₙ = 1 - Eₘₐₓ = -262142
    /// * s ∈ {0, 1}
    /// * Eₘᵢₙ - p + 1 <= t <= Eₘₐₓ - p + 1
    /// * 0 <= c < 2ᵖ
    ///
    /// so that (-1)ˢ × 2ᵗ × c = f.
    #[allow(clippy::cast_possible_wrap)]
    pub(crate) fn decode(&self) -> (u32, i32, u256) {
        debug_assert!(
            self.is_finite(),
            "Attempt to extract sign, exponent and significand from Infinity \
             or NaN."
        );
        // We have a fraction based representation
        // `(-1)ˢ × 2ᵉ × (1 + m × 2¹⁻ᵖ)`, where `Eₘᵢₙ <= e <= Eₘₐₓ` and
        // `0 < m < 2ᵖ⁻¹`
        // or
        // `(-1)ˢ × 2ᵉ × m × 2¹⁻ᵖ`, where `e = Eₘᵢₙ - 1` and
        // `0 < m < 2ᵖ⁻¹`
        // and need to transform it into an integer based representation
        // `(-1)ˢ × 2ᵗ × c`.
        if self.is_zero() {
            return (self.sign(), 0, u256::default());
        }
        let mut c = self.significand();
        let mut t = self.exponent();
        let ntz = c.trailing_zeros();
        c >>= ntz;
        t += ntz as i32;
        (self.sign(), t, c)
    }

    /// Only public for testing!!!
    #[doc(hidden)]
    #[must_use]
    pub fn as_sign_exp_signif(&self) -> (u32, i32, (u128, u128)) {
        let (s, t, c) = self.decode();
        (s, t, (c.hi, c.lo))
    }

    /// Returns `true` if this value is `NaN`.
    #[must_use]
    #[inline]
    pub const fn is_nan(self) -> bool {
        ((self.bits.hi & HI_ABS_MASK) | (self.bits.lo != 0) as u128)
            > HI_EXP_MASK
    }

    /// Returns `true` if this value is positive infinity or negative infinity,
    /// and `false` otherwise.
    #[must_use]
    #[inline]
    pub const fn is_infinite(self) -> bool {
        (self.bits.hi & HI_ABS_MASK) == HI_EXP_MASK && self.bits.lo == 0
    }

    /// Returns `true` if this number is neither infinite nor NaN.
    #[must_use]
    #[inline]
    pub const fn is_finite(self) -> bool {
        (self.bits.hi & HI_EXP_MASK) != HI_EXP_MASK
    }

    /// Returns `true` if the number is subnormal.
    #[must_use]
    #[inline]
    pub const fn is_subnormal(self) -> bool {
        (self.bits.hi & HI_EXP_MASK) == 0 && !self.is_zero()
    }

    /// Returns `true` if the number is neither zero, infinite, subnormal, or
    /// NaN.
    #[must_use]
    #[inline]
    pub const fn is_normal(self) -> bool {
        self.biased_exponent().wrapping_sub(1) < EXP_MAX - 1
    }

    /// Returns the floating point category of the number. If only one property
    /// is going to be tested, it is generally faster to use the specific
    /// predicate instead.
    #[inline]
    #[must_use]
    pub const fn classify(&self) -> FpCategory {
        match (
            self.bits.hi & HI_EXP_MASK,
            self.bits.hi & HI_FRACTION_MASK,
            self.bits.lo,
        ) {
            (HI_EXP_MASK, 0, 0) => FpCategory::Infinite,
            (HI_EXP_MASK, ..) => FpCategory::Nan,
            (0, 0, 0) => FpCategory::Zero,
            (0, ..) => FpCategory::Subnormal,
            _ => FpCategory::Normal,
        }
    }

    /// Returns `true` if `self` is equal to `+0.0` or `-0.0`.
    #[must_use]
    #[inline]
    pub const fn is_zero(self) -> bool {
        (self.bits.hi << 1) == 0 && self.bits.lo == 0
    }

    /// Returns `true` if `self` is either not a number, infinite or equal to
    /// zero.
    #[must_use]
    #[inline]
    pub const fn is_special(self) -> bool {
        (self.bits.hi & HI_ABS_MASK | (self.bits.lo != 0) as u128)
            .wrapping_sub(1)
            >= MAX_HI
    }

    /// Returns `true` if `self` has a positive sign, including `+0.0`, positive
    /// infinity and NaN.
    #[must_use]
    #[inline]
    pub const fn is_sign_positive(self) -> bool {
        self.bits.hi < HI_SIGN_MASK
    }

    /// Returns `true` if `self` has a negative sign, including `-0.0` and
    /// negative infinity.
    #[must_use]
    #[inline]
    pub const fn is_sign_negative(self) -> bool {
        self.bits.hi >= HI_SIGN_MASK
    }

    /// Returns the reciprocal (multiplicative inverse) of `self`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use f256::f256;
    /// let f = f256::from(16);
    /// let r = f256::from(0.0625);
    /// assert_eq!(f.recip(), r);
    ///
    /// assert_eq!(f256::INFINITY.recip(), f256::ZERO);
    /// assert_eq!(f256::NEG_ZERO.recip(), f256::NEG_INFINITY);
    /// assert!(f256::NAN.recip().is_nan());
    /// ```
    #[must_use]
    #[inline]
    pub fn recip(self) -> Self {
        Self::ONE / self
    }

    /// Converts radians to degrees.
    ///
    /// Returns self * (180 / π)
    #[must_use]
    #[inline]
    pub fn to_degrees(self) -> Self {
        // 1 rad = 180 / π ≅ M / 2²⁵⁰
        const M: u256 = u256::new(
            304636616676435425756912514760952666071,
            69798147688063442975655060594812004816,
        );
        const SH: i32 = 250_i32;
        let signif = self.significand();
        let exp = self.exponent();
        let mut t = M.widening_mul(&signif);
        let sh = signif.msb() + 256 - SIGNIFICAND_BITS;
        t.idiv_pow2(sh);
        Self::encode(self.sign(), exp - SH + sh as i32, t.lo)
    }

    /// Converts degrees to radians.
    ///
    /// Returns self * (π / 180)
    #[must_use]
    #[inline]
    pub fn to_radians(self) -> Self {
        // π / 180 ≅ M / 2²⁶¹
        const M: u256 = u256::new(
            190049526055994088508387621895443694809,
            953738875812114979603059177117484306,
        );
        const SH: i32 = 261_i32;
        let signif = self.significand();
        let exp = self.exponent();
        let mut t = M.widening_mul(&signif);
        let sh = signif.msb() + 256 - SIGNIFICAND_BITS;
        t.idiv_pow2(sh);
        Self::encode(self.sign(), exp - SH + sh as i32, t.lo)
    }

    /// Returns the maximum of the two numbers, ignoring NaN.
    ///
    /// If one of the arguments is NaN, then the other argument is returned.
    /// This follows the IEEE-754 2008 semantics for maxNum, except for handling
    /// of signaling NaNs; this function handles all NaNs the same way and
    /// avoids maxNum's problems with associativity.
    #[must_use]
    #[inline]
    pub fn max(self, other: Self) -> Self {
        if other > self || self.is_nan() {
            return other;
        }
        self
    }

    /// Returns the minimum of the two numbers, ignoring NaN.
    ///
    /// If one of the arguments is NaN, then the other argument is returned.
    /// This follows the IEEE-754 2008 semantics for minNum, except for handling
    /// of signaling NaNs; this function handles all NaNs the same way and
    /// avoids minNum's problems with associativity.
    #[must_use]
    #[inline]
    pub fn min(self, other: Self) -> Self {
        if other < self || self.is_nan() {
            return other;
        }
        self
    }

    /// Raw transmutation to `[u64; 4]` (in native endian order).
    #[inline]
    #[must_use]
    pub const fn to_bits(&self) -> [u64; 4] {
        self.bits.to_bits()
    }

    /// Raw transmutation from `[u64; 4]` (in native endian order).
    #[inline]
    #[must_use]
    pub const fn from_bits(bits: [u64; 4]) -> Self {
        Self {
            bits: u256::from_bits(bits),
        }
    }

    /// Return the memory representation of this floating point number as a byte
    /// array in big-endian (network) byte order.
    #[must_use]
    #[inline]
    #[allow(unsafe_code)]
    pub const fn to_be_bytes(self) -> [u8; 32] {
        let bytes = [self.bits.hi.to_be_bytes(), self.bits.lo.to_be_bytes()];
        // SAFETY: safe because size of [[u8; 16]; 2] == size of [u8; 32]
        unsafe { core::mem::transmute(bytes) }
    }

    /// Return the memory representation of this floating point number as a byte
    /// array in little-endian byte order.
    #[must_use]
    #[inline]
    #[allow(unsafe_code)]
    pub const fn to_le_bytes(self) -> [u8; 32] {
        let bytes = [self.bits.lo.to_le_bytes(), self.bits.hi.to_le_bytes()];
        // SAFETY: safe because size of [[u8; 16]; 2] == size of [u8; 32]
        unsafe { core::mem::transmute(bytes) }
    }

    /// Return the memory representation of this floating point number as a byte
    /// array in native byte order.
    #[must_use]
    #[inline]
    #[allow(unsafe_code)]
    pub const fn to_ne_bytes(self) -> [u8; 32] {
        let bytes = self.to_bits();
        // SAFETY: safe because size of [u64; 4] == size of [u8; 32]
        unsafe { core::mem::transmute(bytes) }
    }

    /// Create a floating point value from its representation as a byte array in
    /// big endian.
    #[must_use]
    #[inline]
    #[allow(unsafe_code)]
    pub const fn from_be_bytes(bytes: [u8; 32]) -> Self {
        // SAFETY: safe because size of [[u8; 16]; 2] == size of [u8; 32]
        let bits: [[u8; 16]; 2] = unsafe { core::mem::transmute(bytes) };
        Self {
            bits: u256 {
                hi: u128::from_be_bytes(bits[0]),
                lo: u128::from_be_bytes(bits[1]),
            },
        }
    }

    /// Create a floating point value from its representation as a byte array in
    /// little endian.
    #[must_use]
    #[inline]
    #[allow(unsafe_code)]
    pub const fn from_le_bytes(bytes: [u8; 32]) -> Self {
        // SAFETY: safe because size of [[u8; 16]; 2] == size of [u8; 32]
        let bits: [[u8; 16]; 2] = unsafe { core::mem::transmute(bytes) };
        Self {
            bits: u256 {
                hi: u128::from_le_bytes(bits[1]),
                lo: u128::from_le_bytes(bits[0]),
            },
        }
    }

    /// Create a floating point value from its representation as a byte array in
    /// native endian.
    #[must_use]
    #[inline]
    #[allow(unsafe_code)]
    pub const fn from_ne_bytes(bytes: [u8; 32]) -> Self {
        // SAFETY: safe because size of [[u8; 16]; 2] == size of [u8; 32]
        let bits: [u64; 4] = unsafe { core::mem::transmute(bytes) };
        Self::from_bits(bits)
    }

    /// Return the ordering between `self` and `other`.
    ///
    /// Unlike the standard partial comparison between floating point numbers,
    /// this comparison always produces an ordering in accordance to
    /// the `totalOrder` predicate as defined in the IEEE 754 (2008 revision)
    /// floating point standard. The values are ordered in the following
    /// sequence:
    ///
    /// - negative NaN
    /// - negative infinity
    /// - negative numbers
    /// - negative subnormal numbers
    /// - negative zero
    /// - positive zero
    /// - positive subnormal numbers
    /// - positive numbers
    /// - positive infinity
    /// - positive NaN.
    ///
    /// The ordering established by this function does not always agree with the
    /// [`PartialOrd`] and [`PartialEq`] implementations of `f256`. For example,
    /// they consider negative and positive zero equal, while `total_cmp`
    /// doesn't.
    #[must_use]
    #[inline]
    pub fn total_cmp(&self, other: &Self) -> Ordering {
        // The internal representation of `f256` values gives - besides their
        // sign - a total ordering following the intended mathematical ordering.
        // Thus, flipping the sign bit allows to compare the raw values.
        self.negated().bits.cmp(&(*other).negated().bits)
    }

    /// Restrict a value to a certain interval unless it is NaN.
    ///
    /// Returns `max` if `self` is greater than `max`, and `min` if `self` is
    /// less than `min`. Otherwise this returns `self`.
    ///
    /// Note that this function returns NaN if the initial value was NaN as
    /// well.
    ///
    /// # Panics
    ///
    /// Panics if `min > max`, `min` is NaN, or `max` is NaN.
    ///
    /// # Examples
    ///
    /// ```
    /// # use f256::f256;
    /// let min = f256::EPSILON;
    /// let max = f256::TWO;
    /// let f = f256::ONE;
    /// assert_eq!(f.clamp(min, max), f);
    /// assert_eq!((-f).clamp(min, max), min);
    ///
    /// assert_eq!(f256::INFINITY.clamp(f256::MIN, f256::MAX), f256::MAX);
    /// assert!(f256::NAN.clamp(f256::NEG_INFINITY, f256::INFINITY).is_nan());
    //// ```
    #[must_use]
    #[inline]
    pub fn clamp(self, min: Self, max: Self) -> Self {
        assert!(min <= max);
        if self < min {
            min
        } else if self > max {
            max
        } else {
            self
        }
    }

    /// Computes the absolute value of `self`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use f256::f256;
    /// let f = f256::from(138);
    /// assert_eq!(f.abs(), f);
    /// assert_eq!((-f).abs(), f);
    ///
    /// assert_eq!(f256::MIN.abs(), f256::MAX);
    /// assert_eq!(f256::NEG_INFINITY.abs(), f256::INFINITY);
    /// assert!(f256::NAN.abs().is_nan());
    //// ```
    #[inline(always)]
    #[must_use]
    pub const fn abs(&self) -> Self {
        Self {
            bits: u256 {
                hi: self.bits.hi & HI_ABS_MASK,
                lo: self.bits.lo,
            },
        }
    }

    // Returns the nearest integral value in the direction controlled by the
    // given function.
    #[must_use]
    fn nearest_integral(&self, adj: fn(u32) -> bool) -> Self {
        if self.is_special() {
            return *self;
        }
        // self is finite and non-zero.
        let hi_sign = self.bits.hi & HI_SIGN_MASK;
        let mut abs_bits = u256::new(self.bits.hi & HI_ABS_MASK, self.bits.lo);
        if abs_bits.hi >= MIN_NO_FRACT_HI {
            // |self| >= 2²³⁶, i. e. self is integral.
            return *self;
        }
        let sign = self.sign();
        if abs_bits.hi < ONE.bits.hi {
            // 0 < |self| < 1
            return match (sign, adj(sign)) {
                (0, true) => Self::ONE,
                (1, true) => Self::NEG_ONE,
                (..) => Self::ZERO,
            };
        }
        // 1 < |self| < 2²³⁶
        let n_fract_bits = FRACTION_BITS - (self.biased_exponent() - EXP_BIAS);
        let mut abs_int_bits = &(&abs_bits >> n_fract_bits) << n_fract_bits;
        let c = adj(sign) as u32 * (abs_int_bits != abs_bits) as u32;
        abs_int_bits += &(&u256::new(0, c as u128) << n_fract_bits);
        Self {
            bits: u256::new(abs_int_bits.hi | hi_sign, abs_int_bits.lo),
        }
    }

    /// Returns the integer part of `self`. This means that non-integer numbers
    /// are always truncated towards zero.
    ///
    /// # Examples
    ///
    /// ```
    /// # use f256::f256;
    /// let f = f256::from(177);
    /// let g = f256::from(177.503_f64);
    /// let h = -g;
    ///
    /// assert_eq!(f.trunc(), f);
    /// assert_eq!(g.trunc(), f);
    /// assert_eq!(h.trunc(), -f);
    //// ```
    #[inline]
    #[must_use]
    pub fn trunc(&self) -> Self {
        let adj = |_: u32| false;
        self.nearest_integral(adj)
    }

    /// Returns the fractional part of `self`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use f256::f256;
    /// let f = f256::from(177);
    /// let g = f256::from(177.503_f64);
    /// let h = -g;
    ///
    /// assert_eq!(f.fract(), f256::ZERO);
    /// assert_eq!(g.fract(), g - f);
    /// assert_eq!(h.fract(), f - g);
    //// ```
    #[inline]
    #[must_use]
    pub fn fract(&self) -> Self {
        self - self.trunc()
    }

    /// Returns the integer and the fractional part of `self`.
    #[inline]
    #[must_use]
    pub fn split(&self) -> (Self, Self) {
        let int_part = self.trunc();
        (int_part, self - int_part)
    }

    /// Returns the smallest integer greater than or equal to `self`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use f256::f256;
    /// let f = f256::from(28);
    /// let g = f256::from(27.04_f64);
    /// let h = -g;
    ///
    /// assert_eq!(f.ceil(), f);
    /// assert_eq!(g.ceil(), f);
    /// assert_eq!(h.ceil(), f256::ONE - f);
    //// ```
    #[inline]
    #[must_use]
    pub fn ceil(&self) -> Self {
        let adj = |sign: u32| sign == 0;
        self.nearest_integral(adj)
    }

    /// Returns the largest integer less than or equal to `self`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use f256::f256;
    /// let f = f256::from(57);
    /// let g = f256::from(57.009_f64);
    /// let h = -g;
    ///
    /// assert_eq!(f.floor(), f);
    /// assert_eq!(g.floor(), f);
    /// assert_eq!(h.floor(), -f - f256::ONE);
    //// ```
    #[inline]
    #[must_use]
    pub fn floor(&self) -> Self {
        let adj = |sign: u32| sign == 1;
        self.nearest_integral(adj)
    }

    /// Returns the nearest integer to `self`. Rounds half-way cases away from
    /// zero.
    ///
    /// # Examples
    ///
    /// ```
    /// # use f256::f256;
    /// let f = f256::from(28);
    /// let g = f256::from(27.704_f64);
    /// let h = f256::from(28.5_f64);;
    /// assert_eq!(f.round(), f);
    /// assert_eq!(g.round(), f);
    /// assert_eq!(h.round(), f);
    //// ```
    #[must_use]
    pub fn round(&self) -> Self {
        if self.is_special() {
            return *self;
        }
        // self is finite and non-zero.
        let hi_sign = self.bits.hi & HI_SIGN_MASK;
        let mut abs_bits = u256::new(self.bits.hi & HI_ABS_MASK, self.bits.lo);
        if abs_bits.hi >= MIN_NO_FRACT_HI {
            // |self| >= 2²³⁶, i. e. self is integral.
            return *self;
        }
        if abs_bits.hi <= ONE_HALF.bits.hi {
            // 0 < |self| <= ½
            return Self::ZERO;
        }
        if abs_bits.hi <= ONE.bits.hi {
            // ½ < |self| <= 1
            return Self {
                bits: u256::new(ONE.bits.hi | hi_sign, 0),
            };
        }
        // 1 < |self| < 2²³⁶
        let n_fract_bits = FRACTION_BITS - (self.biased_exponent() - EXP_BIAS);
        abs_bits.idiv_pow2(n_fract_bits);
        abs_bits <<= n_fract_bits;
        Self {
            bits: u256::new(abs_bits.hi | hi_sign, abs_bits.lo),
        }
    }

    /// Returns the additive inverse of `self`.
    #[inline(always)]
    pub(crate) const fn negated(&self) -> Self {
        Self {
            bits: u256 {
                hi: self.bits.hi ^ HI_SIGN_MASK,
                lo: self.bits.lo,
            },
        }
    }
}

impl Neg for f256 {
    type Output = Self;

    #[inline(always)]
    fn neg(self) -> Self::Output {
        self.negated()
    }
}

impl Neg for &f256 {
    type Output = <f256 as Neg>::Output;

    #[inline(always)]
    fn neg(self) -> Self::Output {
        self.negated()
    }
}

#[cfg(test)]
mod repr_tests {
    use super::*;

    #[test]
    fn test_zero() {
        let z = f256::ZERO;
        assert_eq!(z.sign(), 0);
        assert_eq!(z.exponent(), 0);
        assert_eq!(z.significand(), u256::default());
        assert_eq!(z.decode(), (0, 0, u256::default()));
        let z = f256::NEG_ZERO;
        assert_eq!(z.sign(), 1);
        assert_eq!(z.exponent(), 0);
        assert_eq!(z.significand(), u256::default());
        assert_eq!(z.decode(), (1, 0, u256::default()));
    }

    #[test]
    fn test_one() {
        let i = f256::ONE;
        assert_eq!(i.sign(), 0);
        assert_eq!(i.biased_exponent(), EXP_BIAS);
        assert_eq!(i.exponent(), INT_EXP);
        assert_eq!(
            i.significand(),
            u256 {
                hi: 1_u128 << HI_FRACTION_BITS,
                lo: 0
            }
        );
        assert_eq!(i.decode(), (0, 0, u256 { hi: 0, lo: 1 }));
        let j = f256::NEG_ONE;
        assert_eq!(j.sign(), 1);
        assert_eq!(i.biased_exponent(), EXP_BIAS);
        assert_eq!(j.exponent(), INT_EXP);
        assert_eq!(
            j.significand(),
            u256 {
                hi: 1_u128 << HI_FRACTION_BITS,
                lo: 0
            }
        );
        assert_eq!(j.decode(), (1, 0, u256 { hi: 0, lo: 1 }));
    }

    #[test]
    fn test_normal() {
        let i = f256::TWO;
        assert_eq!(i.sign(), 0);
        assert_eq!(i.biased_exponent(), EXP_BIAS + 1);
        assert_eq!(i.exponent(), INT_EXP + 1);
        assert_eq!(
            i.significand(),
            u256 {
                hi: 1_u128 << HI_FRACTION_BITS,
                lo: 0
            }
        );
        assert_eq!(i.decode(), (0, 1, u256 { hi: 0, lo: 1 }));
        let f = f256::from(-3.5_f64);
        assert_eq!(f.sign(), 1);
        assert_eq!(f.exponent(), -235);
        assert_eq!(
            f.significand(),
            u256 {
                hi: 567907468902246771870523036008448,
                lo: 0
            }
        );
        assert_eq!(f.decode(), (1, -1, u256 { hi: 0, lo: 7 }));
    }

    #[test]
    #[allow(clippy::cast_possible_wrap)]
    fn test_subnormal() {
        let f = f256::MIN_GT_ZERO;
        assert_eq!(f.sign(), 0);
        assert_eq!(f.exponent(), EMIN - FRACTION_BITS as i32);
        assert_eq!(f.significand(), u256 { hi: 0, lo: 1 });
        assert_eq!(
            f.decode(),
            (0, EMIN - FRACTION_BITS as i32, u256 { hi: 0, lo: 1 })
        );
    }
}

#[cfg(test)]
mod encode_decode_tests {
    use super::*;

    #[test]
    fn test_normal() {
        let sign = 1_u32;
        let exponent = -23_i32;
        let significand = u256 {
            hi: 39,
            lo: 10000730744,
        };
        let f = f256::encode(sign, exponent, significand);
        let (s, t, c) = f.decode();
        let g = f256::encode(s, t, c);
        assert_eq!(f, g);
    }

    #[test]
    fn test_subnormal() {
        let sign = 0_u32;
        let exponent = EMIN - 235_i32;
        let significand = u256 {
            hi: u128::MAX >> (EXP_BITS + 2),
            lo: 0,
        };
        let f = f256::encode(sign, exponent, significand);
        assert!(f.is_subnormal());
        let (s, t, c) = f.decode();
        let g = f256::encode(s, t, c);
        assert_eq!(f, g);
        let f = f256::MIN_GT_ZERO;
        let (s, t, c) = f.decode();
        let g = f256::encode(s, t, c);
        assert_eq!(f, g);
    }
}

#[cfg(test)]
mod split_tests {
    use super::*;

    #[test]
    fn test_normal() {
        let f = f256::from(17);
        let g = f256::from(17.625_f64);
        let h = -g;
        let (fi, ff) = f.split();
        assert_eq!(fi, f);
        assert_eq!(fi.exponent(), f.exponent());
        assert_eq!(ff, f256::ZERO);
        let (gi, gf) = g.split();
        assert_eq!(gi, f);
        assert_eq!(gi.exponent(), g.exponent());
        assert_eq!(gf, g - f);
        assert_eq!(h.split(), (-f, (f - g)));
    }

    #[test]
    fn test_lt_1() {
        let f = f256::from(0.99999_f64);
        let (fi, ff) = f.split();
        assert_eq!(fi, f256::ZERO);
        assert_eq!(fi.exponent(), 0);
        assert_eq!(ff, f);
        assert_eq!(ff.exponent(), f.exponent());
    }

    #[test]
    fn test_subnormal() {
        let f = f256::MIN_GT_ZERO;
        assert_eq!(f.split(), (f256::ZERO, f256::MIN_GT_ZERO));
    }
}
