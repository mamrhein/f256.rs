// ---------------------------------------------------------------------------
// Copyright:   (c) 2021 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

#![doc = include_str!("../README.md")]
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

extern crate alloc;
extern crate core;

use core::{cmp::Ordering, convert::Into, num::FpCategory, ops::Neg};

use crate::big_uint::{BigUInt, DivRem, HiLo, U1024, U128, U256, U512};

mod big_uint;
mod binops;
pub mod consts;
mod conv;
mod fused_ops;
mod math;
#[cfg(feature = "num-traits")]
mod num_traits;

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
    (1_u128 << (u128::BITS - 1)) - (1_u128 << HI_FRACTION_BITS) - 1;
/// Binary exponent for integral values
#[allow(clippy::cast_possible_wrap)]
pub(crate) const INT_EXP: i32 = -(FRACTION_BITS as i32);
/// Value of hi u128 of the smallest f256 value with no fractional part (2²³⁶)
pub(crate) const MIN_NO_FRACT_HI: u128 =
    ((EXP_BIAS + FRACTION_BITS) as u128) << HI_FRACTION_BITS;
/// Minimum possible subnormal power of 10 exponent =
/// ⌊(Eₘᵢₙ + 1 - p) × log₁₀(2)⌋.
pub(crate) const MIN_GT_ZERO_10_EXP: i32 = -78984;

/// A 256-bit floating point type (specifically, the “binary256” type defined
/// in IEEE 754-2008).
///
/// For details see [above](index.html).
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Default)]
pub struct f256 {
    pub(crate) bits: U256,
}

/// Some f256 constants (only used to hide the internals in the doc)
const EPSILON: f256 = f256 {
    bits: U256::new(EPSILON_HI, 0),
};
const MAX: f256 = f256 {
    bits: U256::new(MAX_HI, u128::MAX),
};
const MIN: f256 = MAX.negated();
const MIN_POSITIVE: f256 = f256 {
    bits: U256::new(HI_FRACTION_BIAS, 0),
};
const MIN_GT_ZERO: f256 = f256 { bits: U256::ONE };
const NAN: f256 = f256 {
    bits: U256::new(NAN_HI, 0),
};
const INFINITY: f256 = f256 {
    bits: U256::new(INF_HI, 0),
};
const NEG_INFINITY: f256 = f256 {
    bits: U256::new(NEG_INF_HI, 0),
};
const ZERO: f256 = f256 { bits: U256::ZERO };
const NEG_ZERO: f256 = ZERO.negated();
pub(crate) const ONE_HALF: f256 = f256 {
    bits: U256::new((((EXP_BIAS - 1) as u128) << HI_FRACTION_BITS), 0),
};
const ONE: f256 = f256 {
    bits: U256::new(((EXP_BIAS as u128) << HI_FRACTION_BITS), 0),
};
const NEG_ONE: f256 = ONE.negated();
const TWO: f256 = f256 {
    bits: U256::new((((1 + EXP_BIAS) as u128) << HI_FRACTION_BITS), 0),
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
    /// Note that IEEE-745 doesn't define just a single NaN value; a plethora
    /// of bit patterns are considered to be NaN. Furthermore, the
    /// standard makes a difference between a "signaling" and a "quiet"
    /// NaN, and allows inspecting its "payload" (the unspecified bits in
    /// the bit pattern). This implementation does not make such a
    /// difference and uses exactly one bit pattern for NaN.
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

    /// Raw assembly from sign, exponent and significand.
    #[inline]
    pub(crate) const fn new(
        sign: u32,
        exponent: i32,
        significand: U256,
    ) -> Self {
        debug_assert!(sign == 0 || sign == 1);
        debug_assert!(exponent >= EMIN - 1 && exponent <= EMAX);
        debug_assert!(!significand.is_zero());
        debug_assert!((significand.hi.0 >> HI_FRACTION_BITS) <= 1_u128);
        let biased_exp = (exponent + EXP_BIAS as i32) as u128;
        Self {
            bits: U256::new(
                (significand.hi.0 & HI_FRACTION_MASK)
                    | (biased_exp << HI_FRACTION_BITS)
                    | ((sign as u128) << HI_SIGN_SHIFT),
                significand.lo.0,
            ),
        }
    }

    /// Construct a finite, non-zero `f256` value f from sign s, quantum
    /// exponent t and integral significand c,
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
    pub(crate) fn encode(s: u32, mut t: i32, mut c: U256) -> Self {
        debug_assert!(!c.is_zero());
        // We have an integer based representation `(-1)ˢ × 2ᵗ × c` and need
        // to transform it into a fraction based representation
        // `(-1)ˢ × 2ᵉ × (1 + m × 2¹⁻ᵖ)`,
        // where `Eₘᵢₙ <= e <= Eₘₐₓ` and `0 < m < 2ᵖ⁻¹`, or
        // `(-1)ˢ × 2ᵉ × m × 2¹⁻ᵖ`,
        // where `e = Eₘᵢₙ - 1` and `0 < m < 2ᵖ⁻¹`.

        // 1. Compensate radix shift
        t += FRACTION_BITS as i32;
        // 2. Normalize significand
        let nlz = c.leading_zeros();
        // The position of the most significant bit is `256 - nlz - 1`. We
        // need to shift it to the position of the hidden bit, which
        // is `256 - EXP_BITS - 1`. So we have to shift by |nlz -
        // EXP_BITS|.
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
                let mut shift = (EXP_BITS - nlz);
                t += shift as i32;
                c = c.rounding_div_pow2(shift);
                // Rounding may have caused significand to overflow.
                if (c.hi.0 >> (HI_FRACTION_BITS + 1)) != 0 {
                    t += 1;
                    c >>= 1;
                }
            }
            _ => {}
        }
        debug_assert!(
            (EMIN - 1..=EMAX).contains(&t),
            "Exponent limits exceeded: {t}"
        );
        // 3. Assemble struct f256
        Self::new(s, t, c)
    }

    /// Only public for testing!!!
    #[doc(hidden)]
    #[must_use]
    pub fn from_sign_exp_signif(s: u32, t: i32, c: (u128, u128)) -> Self {
        debug_assert!(s == 0 || s == 1);
        let c = U256::new(c.0, c.1);
        if c.is_zero() {
            if t == 0 {
                return [Self::ZERO, Self::NEG_ZERO][s as usize];
            }
            if t == EMAX + 1 {
                return [Self::INFINITY, Self::NEG_INFINITY][s as usize];
            }
        }
        Self::encode(s, t, c)
    }

    /// Returns the sign bit of `self`: 0 = positive, 1 = negative.
    #[inline]
    pub(crate) const fn sign(&self) -> u32 {
        let self1 = &self.bits;
        (self1.hi.0 >> HI_SIGN_SHIFT) as u32
    }

    /// Returns the biased binary exponent of `self`.
    #[inline]
    pub(crate) const fn biased_exponent(&self) -> u32 {
        ((self.bits.hi.0 & HI_EXP_MASK) >> HI_FRACTION_BITS) as u32
    }

    /// Returns the quantum exponent of `self`.
    /// Pre-condition: `self` is finite!
    #[inline]
    #[allow(clippy::cast_possible_wrap)]
    pub(crate) const fn quantum_exponent(&self) -> i32 {
        debug_assert!(
            self.is_finite(),
            "Attempt to extract quantum exponent from Infinity or NaN."
        );
        const TOTAL_BIAS: i32 = EXP_BIAS as i32 + FRACTION_BITS as i32;
        let mut exp = self.biased_exponent() as i32;
        exp += (exp == 0) as i32; // Adjust exp for subnormals.
        (!self.eq_zero() as i32) * (exp - TOTAL_BIAS)
    }

    /// Returns the exponent of `self`.
    /// Pre-condition: `self` is finite!
    #[inline]
    #[allow(clippy::cast_possible_wrap)]
    pub(crate) const fn exponent(&self) -> i32 {
        debug_assert!(
            self.is_finite(),
            "Attempt to extract exponent from Infinity or NaN."
        );
        let mut exp = self.biased_exponent() as i32;
        exp += (exp == 0) as i32; // Adjust exp for subnormals.
        (!self.eq_zero() as i32) * (exp - EXP_BIAS as i32)
    }

    /// Returns the fraction of `self`.
    #[inline]
    pub(crate) const fn fraction(&self) -> U256 {
        U256::new(self.bits.hi.0 & HI_FRACTION_MASK, self.bits.lo.0)
    }

    /// Returns the integral significand of `self`.
    /// Pre-condition: `self` is finite!
    #[inline]
    pub(crate) const fn integral_significand(&self) -> U256 {
        debug_assert!(
            self.is_finite(),
            "Attempt to extract integral significand from Infinity or NaN."
        );
        let hidden_one =
            ((self.biased_exponent() != 0) as u128) << HI_FRACTION_BITS;
        U256::new(
            (self.bits.hi.0 & HI_FRACTION_MASK) | hidden_one,
            self.bits.lo.0,
        )
    }

    /// Returns the significand of `self`.
    /// Pre-condition: `self` is finite!
    pub(crate) fn significand(&self) -> Self {
        debug_assert!(
            self.is_finite(),
            "Attempt to extract significand from Infinity or NaN."
        );
        if self.eq_zero() {
            return *self;
        }
        let mut biased_exp = EXP_BIAS;
        let mut bits =
            U256::new((self.bits.hi.0 & HI_FRACTION_MASK), self.bits.lo.0);
        if self.biased_exponent() == 0 {
            // self is subnormal
            let shift = (bits.leading_zeros() - EXP_BITS);
            bits = bits.shift_left(shift);
            biased_exp -= shift + 1;
        }
        bits.hi.0 += (biased_exp as u128) << HI_FRACTION_BITS;
        Self { bits }
    }

    /// Extract sign s, quantum exponent t and integral significand c from
    /// a finite `f256` f,
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
    pub(crate) fn decode(&self) -> (u32, i32, U256) {
        debug_assert!(
            self.is_finite(),
            "Attempt to extract sign, exponent and significand from \
             Infinity or NaN."
        );
        // We have a fraction based representation
        // `(-1)ˢ × 2ᵉ × (1 + m × 2¹⁻ᵖ)`, where `Eₘᵢₙ <= e <= Eₘₐₓ` and
        // `0 < m < 2ᵖ⁻¹`
        // or
        // `(-1)ˢ × 2ᵉ × m × 2¹⁻ᵖ`, where `e = Eₘᵢₙ - 1` and
        // `0 < m < 2ᵖ⁻¹`
        // and need to transform it into an integer based representation
        // `(-1)ˢ × 2ᵗ × c`.
        let (s, mut t, mut c) = split_f256_enc(self);
        if !c.is_zero() {
            let ntz = c.trailing_zeros();
            c >>= ntz;
            t += ntz as i32;
        }
        (s, t, c)
    }

    /// Only public for testing!!!
    #[doc(hidden)]
    #[must_use]
    pub fn as_sign_exp_signif(&self) -> (u32, i32, (u128, u128)) {
        let (s, t, c) = self.decode();
        (s, t, (c.hi.0, c.lo.0))
    }

    /// Returns `true` if this value is `NaN`.
    #[must_use]
    #[inline]
    pub const fn is_nan(self) -> bool {
        ((self.bits.hi.0 & HI_ABS_MASK) | (self.bits.lo.0 != 0) as u128)
            > HI_EXP_MASK
    }

    /// Returns `true` if this value is positive infinity or negative
    /// infinity, and `false` otherwise.
    #[must_use]
    #[inline]
    pub const fn is_infinite(self) -> bool {
        (self.bits.hi.0 & HI_ABS_MASK) == HI_EXP_MASK && self.bits.lo.0 == 0
    }

    /// Returns `true` if this number is neither infinite nor NaN.
    #[must_use]
    #[inline]
    pub const fn is_finite(self) -> bool {
        (self.bits.hi.0 & HI_EXP_MASK) != HI_EXP_MASK
    }

    /// Returns `true` if the number is subnormal.
    #[must_use]
    #[inline]
    pub const fn is_subnormal(self) -> bool {
        (self.bits.hi.0 & HI_EXP_MASK) == 0 && !self.eq_zero()
    }

    /// Returns `true` if the number is neither zero, infinite, subnormal, or
    /// NaN.
    #[must_use]
    #[inline]
    pub const fn is_normal(self) -> bool {
        self.biased_exponent().wrapping_sub(1) < EXP_MAX - 1
    }

    /// Returns the floating point category of the number. If only one
    /// property is going to be tested, it is generally faster to use the
    /// specific predicate instead.
    #[inline]
    #[must_use]
    pub const fn classify(&self) -> FpCategory {
        let abs_bits_sticky = abs_bits_sticky(&abs_bits(self));
        match abs_bits_sticky {
            0 => FpCategory::Zero,
            INF_HI => FpCategory::Infinite,
            NAN_HI => FpCategory::Nan,
            ..=HI_FRACTION_MASK => FpCategory::Subnormal,
            _ => FpCategory::Normal,
        }
    }

    /// Returns `true` if `self` is equal to `+0.0` or `-0.0`.
    #[must_use]
    #[inline]
    pub const fn eq_zero(self) -> bool {
        (self.bits.hi.0 << 1) == 0 && self.bits.lo.0 == 0
    }

    /// Returns `true` if `self` is either not a number, infinite or equal to
    /// zero.
    #[must_use]
    #[inline]
    pub const fn is_special(self) -> bool {
        (self.bits.hi.0 & HI_ABS_MASK | (self.bits.lo.0 != 0) as u128)
            .wrapping_sub(1)
            >= MAX_HI
    }

    /// Returns `true` if `self` has a positive sign, including `+0.0`,
    /// positive infinity and NaN.
    #[must_use]
    #[inline]
    pub const fn is_sign_positive(self) -> bool {
        self.bits.hi.0 < HI_SIGN_MASK
    }

    /// Returns `true` if `self` has a negative sign, including `-0.0` and
    /// negative infinity.
    #[must_use]
    #[inline]
    pub const fn is_sign_negative(self) -> bool {
        self.bits.hi.0 >= HI_SIGN_MASK
    }

    /// Returns the unit in the last place of `self`.
    ///
    /// ULP denotes the magnitude of the last significand digit of `self`.
    #[must_use]
    pub fn ulp(&self) -> Self {
        let abs_bits_self = abs_bits(self);
        let mut exp_bits = exp_bits(&abs_bits_self);
        if exp_bits < EXP_MAX {
            // `self` is finite.
            let mut bits = U256::new(HI_FRACTION_BIAS, 0_u128);
            let sh = FRACTION_BITS
                .saturating_sub(exp_bits - norm_bit(&abs_bits_self));
            exp_bits = exp_bits.saturating_sub(FRACTION_BITS + 1);
            bits >>= sh;
            bits.hi.0 += (exp_bits as u128) << HI_FRACTION_BITS;
            f256 { bits }
        } else {
            // `self` is infinite or nan.
            NAN
        }
    }

    /// Returns true, if |self - other| <= N * self.ulp()
    #[must_use]
    #[inline]
    pub(crate) fn almost_eq<const N: u8>(&self, other: &Self) -> bool {
        (self - other).abs() <= Self::from(N) * self.ulp()
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
    #[allow(clippy::cast_possible_wrap)]
    pub fn to_degrees(self) -> Self {
        // 1 rad = 180 / π ≅ M / 2²⁵⁰
        const M: U256 = U256::new(
            304636616676435425756912514760952666071,
            69798147688063442975655060594812004816,
        );
        const SH: i32 = 250_i32;
        let signif = self.integral_significand();
        let exp = self.quantum_exponent();
        let (lo, hi) = M.widening_mul(&signif);
        let mut t = U512::from_hi_lo(hi, lo);
        let sh = signif.msb() + 256 - SIGNIFICAND_BITS;
        t = t.rounding_div_pow2(sh);
        Self::encode(self.sign(), exp - SH + sh as i32, t.lo)
    }

    /// Converts degrees to radians.
    ///
    /// Returns self * (π / 180)
    #[must_use]
    #[inline]
    #[allow(clippy::cast_possible_wrap)]
    pub fn to_radians(self) -> Self {
        // π / 180 ≅ M / 2²⁶¹
        const M: U256 = U256::new(
            190049526055994088508387621895443694809,
            953738875812114979603059177117484306,
        );
        const SH: i32 = 261_i32;
        let signif = self.integral_significand();
        let exp = self.quantum_exponent();
        let (lo, hi) = M.widening_mul(&signif);
        let mut t = U512::from_hi_lo(hi, lo);
        let sh = signif.msb() + 256 - SIGNIFICAND_BITS;
        t = t.rounding_div_pow2(sh);
        Self::encode(self.sign(), exp - SH + sh as i32, t.lo)
    }

    /// Returns the maximum of the two numbers, ignoring NaN.
    ///
    /// If one of the arguments is NaN, then the other argument is returned.
    /// This follows the IEEE-754 2008 semantics for maxNum, except for
    /// handling of signaling NaNs; this function handles all NaNs the
    /// same way and avoids maxNum's problems with associativity.
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
    /// This follows the IEEE-754 2008 semantics for minNum, except for
    /// handling of signaling NaNs; this function handles all NaNs the
    /// same way and avoids minNum's problems with associativity.
    #[must_use]
    #[inline]
    pub fn min(self, other: Self) -> Self {
        if other < self || self.is_nan() {
            return other;
        }
        self
    }

    /// Raw transmutation to `(u128, u128)` ((self.bits.hi.0, self.bits.lo.0),
    /// each in native endian order).
    #[inline]
    #[must_use]
    pub const fn to_bits(&self) -> (u128, u128) {
        (self.bits.hi.0, self.bits.lo.0)
    }

    /// Raw transmutation from `(u128, u128)` ((self.bits.hi.0,
    /// self.bits.lo.0), each in native endian order).
    #[inline]
    #[must_use]
    pub const fn from_bits(bits: (u128, u128)) -> Self {
        Self {
            bits: U256::new(bits.0, bits.1),
        }
    }

    /// Return the memory representation of this floating point number as a
    /// byte array in big-endian (network) byte order.
    #[must_use]
    #[inline]
    #[allow(unsafe_code)]
    pub const fn to_be_bytes(self) -> [u8; 32] {
        let bytes =
            [self.bits.hi.0.to_be_bytes(), self.bits.lo.0.to_be_bytes()];
        // SAFETY: safe because size of [[u8; 16]; 2] == size of [u8; 32]
        unsafe { core::mem::transmute(bytes) }
    }

    /// Return the memory representation of this floating point number as a
    /// byte array in little-endian byte order.
    #[must_use]
    #[inline]
    #[allow(unsafe_code)]
    pub const fn to_le_bytes(self) -> [u8; 32] {
        let bytes =
            [self.bits.lo.0.to_le_bytes(), self.bits.hi.0.to_le_bytes()];
        // SAFETY: safe because size of [[u8; 16]; 2] == size of [u8; 32]
        unsafe { core::mem::transmute(bytes) }
    }

    /// Return the memory representation of this floating point number as a
    /// byte array in native byte order.
    #[must_use]
    #[inline]
    #[allow(unsafe_code)]
    pub const fn to_ne_bytes(self) -> [u8; 32] {
        let bits = self.to_bits();
        // SAFETY: safe because size of (u128, u128) == size of [u8; 32]
        unsafe { core::mem::transmute(bits) }
    }

    /// Create a floating point value from its representation as a byte array
    /// in big endian.
    #[must_use]
    #[inline]
    #[allow(unsafe_code)]
    pub const fn from_be_bytes(bytes: [u8; 32]) -> Self {
        // SAFETY: safe because size of [[u8; 16]; 2] == size of [u8; 32]
        let bits: [[u8; 16]; 2] = unsafe { core::mem::transmute(bytes) };
        Self {
            bits: U256::new(
                u128::from_be_bytes(bits[0]),
                u128::from_be_bytes(bits[1]),
            ),
        }
    }

    /// Create a floating point value from its representation as a byte array
    /// in little endian.
    #[must_use]
    #[inline]
    #[allow(unsafe_code)]
    pub const fn from_le_bytes(bytes: [u8; 32]) -> Self {
        // SAFETY: safe because size of [[u8; 16]; 2] == size of [u8; 32]
        let bits: [[u8; 16]; 2] = unsafe { core::mem::transmute(bytes) };
        Self {
            bits: U256::new(
                u128::from_le_bytes(bits[1]),
                u128::from_le_bytes(bits[0]),
            ),
        }
    }

    /// Create a floating point value from its representation as a byte array
    /// in native endian.
    #[must_use]
    #[inline]
    #[allow(unsafe_code)]
    pub const fn from_ne_bytes(bytes: [u8; 32]) -> Self {
        // SAFETY: safe because size of (u128, u128) == size of [u8; 32]
        let bits: (u128, u128) = unsafe { core::mem::transmute(bytes) };
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
    /// The ordering established by this function does not always agree with
    /// the [`PartialOrd`] and [`PartialEq`] implementations of `f256`.
    /// For example, they consider negative and positive zero equal, while
    /// `total_cmp` doesn't.
    #[must_use]
    #[inline]
    pub fn total_cmp(&self, other: &Self) -> Ordering {
        // The internal representation of `f256` values gives - besides their
        // sign - a total ordering following the intended mathematical
        // ordering. Thus, flipping the sign bit allows to compare the
        // raw values.
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
            bits: U256::new(self.bits.hi.0 & HI_ABS_MASK, self.bits.lo.0),
        }
    }

    // Returns the nearest integral value in the direction controlled by the
    // given function.
    #[must_use]
    fn nearest_integral(&self, adj: fn(u32) -> bool) -> Self {
        let mut abs_bits = abs_bits(self);
        if abs_bits.is_special() {
            // self is special
            return *self;
        }
        // self is finite and non-zero.
        if abs_bits.hi.0 >= MIN_NO_FRACT_HI {
            // |self| >= 2²³⁶, i. e. self is integral.
            return *self;
        }
        let sign = self.sign();
        if abs_bits.hi.0 < ONE.bits.hi.0 {
            // 0 < |self| < 1
            return match (sign, adj(sign)) {
                (0, true) => Self::ONE,
                (1, true) => Self::NEG_ONE,
                (..) => Self::ZERO,
            };
        }
        // 1 < |self| < 2²³⁶
        let n_fract_bits = FRACTION_BITS - (exp_bits(&abs_bits) - EXP_BIAS);
        let mut abs_int_bits = &(&abs_bits >> n_fract_bits) << n_fract_bits;
        let c = adj(sign) as u32 * (abs_int_bits != abs_bits) as u32;
        abs_int_bits += &(&U256::new(0, c as u128) << n_fract_bits);
        Self {
            bits: U256::new(
                abs_int_bits.hi.0 | sign_bits_hi(self),
                abs_int_bits.lo.0,
            ),
        }
    }

    /// Returns the integer part of `self`. This means that non-integer
    /// numbers are always truncated towards zero.
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
    /// let f = f256::from(27);
    /// let g = f256::from(26.704_f64);
    /// let h = f256::from(26.5_f64);
    /// assert_eq!(f.round(), f);
    /// assert_eq!(g.round(), f);
    /// assert_eq!(h.round(), f);
    //// ```
    #[must_use]
    pub fn round(&self) -> Self {
        let mut abs_bits = abs_bits(self);
        if abs_bits.is_special() {
            // self is special
            return *self;
        }
        // self is finite and non-zero.
        if abs_bits.hi.0 >= MIN_NO_FRACT_HI {
            // |self| >= 2²³⁶, i. e. self is integral.
            return *self;
        }
        if abs_bits.hi.0 < ONE_HALF.bits.hi.0 {
            // 0 < |self| < ½
            return Self::ZERO;
        }
        if abs_bits.hi.0 <= ONE.bits.hi.0 {
            // ½ <= |self| <= 1
            return Self {
                bits: U256::new(ONE.bits.hi.0 | sign_bits_hi(self), 0),
            };
        }
        // 1 < |self| < 2²³⁶
        let n_fract_bits = FRACTION_BITS - (exp_bits(&abs_bits) - EXP_BIAS);
        let tie = &U256::ONE << (n_fract_bits - 1);
        let rem = abs_bits.rem_pow2(n_fract_bits);
        abs_bits >>= n_fract_bits;
        if rem >= tie {
            abs_bits.incr();
        }
        abs_bits <<= n_fract_bits;
        Self {
            bits: U256::new(
                abs_bits.hi.0 | sign_bits_hi(self),
                abs_bits.lo.0,
            ),
        }
    }

    /// Returns the nearest integer to `self`. Rounds half-way cases to the
    /// nearest even.
    ///
    /// # Examples
    ///
    /// ```
    /// # use f256::f256;
    /// let f = f256::from(28);
    /// let g = f256::from(27.704_f64);
    /// let h = f256::from(27.5_f64);
    /// let h = f256::from(28.5_f64);
    /// assert_eq!(f.round_tie_even(), f);
    /// assert_eq!(g.round_tie_even(), f);
    /// assert_eq!(h.round_tie_even(), f);
    //// ```
    #[must_use]
    pub fn round_tie_even(&self) -> Self {
        let mut abs_bits = abs_bits(self);
        if abs_bits.is_special() {
            // self is special
            return *self;
        }
        // self is finite and non-zero.
        if abs_bits.hi.0 >= MIN_NO_FRACT_HI {
            // |self| >= 2²³⁶, i. e. self is integral.
            return *self;
        }
        if abs_bits.hi.0 <= ONE_HALF.bits.hi.0 {
            // 0 < |self| <= ½
            return Self::ZERO;
        }
        if abs_bits.hi.0 <= ONE.bits.hi.0 {
            // ½ < |self| <= 1
            return Self {
                bits: U256::new(ONE.bits.hi.0 | sign_bits_hi(self), 0),
            };
        }
        // 1 < |self| < 2²³⁶
        let n_fract_bits = FRACTION_BITS - (exp_bits(&abs_bits) - EXP_BIAS);
        let mut n = n_fract_bits;
        abs_bits = abs_bits.rounding_div_pow2(n);
        abs_bits <<= n_fract_bits;
        Self {
            bits: U256::new(
                abs_bits.hi.0 | sign_bits_hi(self),
                abs_bits.lo.0,
            ),
        }
    }

    /// Returns the additive inverse of `self`.
    #[inline(always)]
    // TODO: Inline in impl Neg when trait fns can be const.
    pub(crate) const fn negated(&self) -> Self {
        Self {
            bits: U256::new(self.bits.hi.0 ^ HI_SIGN_MASK, self.bits.lo.0),
        }
    }

    /// Returns 2 * `self`
    #[inline(always)]
    pub fn mul2(&self) -> Self {
        self.mul_pow2(1)
    }

    /// Returns `self` * 2ⁿ
    pub fn mul_pow2(&self, n: u32) -> Self {
        let abs_bits = abs_bits(self);
        if abs_bits.is_special() {
            // self is either NaN, infinite or equal 0
            return *self;
        }
        // self is finite and non-zero.
        let exp_bits = exp_bits(&abs_bits);
        if exp_bits.saturating_add(n) >= EXP_MAX {
            return [Self::INFINITY, Self::NEG_INFINITY]
                [self.sign() as usize];
        }
        if exp_bits == 0 {
            // self is subnornal
            const EXP: i32 = 1 - (EXP_BIAS as i32 + FRACTION_BITS as i32);
            let fraction = fraction(&abs_bits);
            let exp = EXP + n as i32;
            return Self::from_sign_exp_signif(
                self.sign(),
                exp,
                (fraction.hi.0, fraction.lo.0),
            );
        }
        // self is normal.
        Self {
            bits: U256::new(
                self.bits.hi.0 + ((n as u128) << HI_FRACTION_BITS),
                self.bits.lo.0,
            ),
        }
    }

    /// Fused multiply-add.
    ///
    /// Computes `(self * f) + a` with only one rounding error, yielding a
    /// more accurate result than a non-fused multiply-add.
    #[inline(always)]
    #[must_use]
    pub fn mul_add(self, f: f256, a: f256) -> Self {
        fused_ops::fma::fma(&self, &f, &a)
    }

    /// Fused sum of squares.
    ///
    /// Computes `(self * self) + (other * other)` with only one rounding
    /// error, yielding a more accurate result than a non-fused sum of
    /// squares.
    #[inline(always)]
    #[must_use]
    pub fn sum_of_squares(self, other: f256) -> Self {
        fused_ops::sos::sos(&self, &other)
    }

    /// Computes `self * self` .
    #[inline(always)]
    #[must_use]
    pub fn square(self) -> Self {
        self * self
    }

    /// Fused square-add.
    ///
    /// Computes `(self * self) + a` with only one rounding error, yielding a
    /// more accurate result than a non-fused square-add.
    #[inline(always)]
    #[must_use]
    pub fn square_add(self, a: f256) -> Self {
        fused_ops::fma::fma(&self, &self, &a)
    }

    /// Returns `self` / 2 (rounded tie to even)
    #[inline(always)]
    pub fn div2(&self) -> Self {
        self.div_pow2(1)
    }

    /// Returns `self` / 2ⁿ (rounded tie to even)
    pub fn div_pow2(&self, n: u32) -> Self {
        let abs_bits = abs_bits(self);
        if abs_bits.is_special() {
            // self is either NaN, infinite or equal 0
            return *self;
        }
        // self is finite and non-zero.
        let exp_bits = exp_bits(&abs_bits);
        if exp_bits < n {
            // result is subnornal or underflows to zero
            const EXP: i32 = 1 - (EXP_BIAS as i32 + FRACTION_BITS as i32);
            let shr = n - exp_bits + norm_bit(&abs_bits);
            if shr > self.bits.msb() {
                return [Self::ZERO, Self::NEG_ZERO][self.sign() as usize];
            }
            let signif = signif(&abs_bits).rounding_div_pow2(shr);
            if signif.is_zero() {
                return [Self::ZERO, Self::NEG_ZERO][self.sign() as usize];
            }
            return Self::from_sign_exp_signif(
                self.sign(),
                EXP,
                (signif.hi.0, signif.lo.0),
            );
        }
        // self is normal.
        Self {
            bits: U256::new(
                self.bits.hi.0 - ((n as u128) << HI_FRACTION_BITS),
                self.bits.lo.0,
            ),
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

// Some helper functions working on the binary encoded representation of an
// f256 value

/// Returns the high bits of f reduced to its sign bit.
#[inline(always)]
pub(crate) const fn sign_bits_hi(f: &f256) -> u128 {
    f.bits.hi.0 & HI_SIGN_MASK
}

/// Returns the representation of f.abs().
#[inline(always)]
pub(crate) const fn abs_bits(f: &f256) -> U256 {
    U256::new(f.bits.hi.0 & HI_ABS_MASK, f.bits.lo.0)
}

/// Returns the high bits of `abs_bits` or'ed with 1 if the lower bits of
/// `abs_bits` != 0.
#[inline(always)]
pub(crate) const fn abs_bits_sticky(abs_bits: &U256) -> u128 {
    abs_bits.hi.0 | (abs_bits.lo.0 != 0) as u128
}

pub(crate) trait BinEncSpecial {
    /// Returns true if self represents |Inf|, NaN or |0|.
    fn is_special(&self) -> bool;
}

impl BinEncSpecial for u128 {
    #[inline(always)]
    fn is_special(&self) -> bool {
        self.wrapping_sub(1) >= MAX_HI
    }
}

impl BinEncSpecial for U256 {
    #[inline(always)]
    fn is_special(&self) -> bool {
        abs_bits_sticky(self).is_special()
    }
}

pub(crate) trait BinEncAnySpecial {
    /// Returns true if any element of self represents |Inf|, NaN or |0|.
    fn any_special(&self) -> bool;

    /// Returns true if any element of self represents a subnormal.
    fn any_subnormal(&self) -> bool;

    /// Returns true if any element of self represents a non-normal.
    fn any_non_normal(&self) -> bool;
}

impl BinEncAnySpecial for (u128, u128) {
    #[inline(always)]
    fn any_special(&self) -> bool {
        self.0.wrapping_sub(1) >= MAX_HI || self.1.wrapping_sub(1) >= MAX_HI
    }

    #[inline(always)]
    fn any_subnormal(&self) -> bool {
        self.0 <= HI_FRACTION_MASK || self.1 <= HI_FRACTION_MASK
    }

    #[inline(always)]
    fn any_non_normal(&self) -> bool {
        self.any_special() || self.any_subnormal()
    }
}

impl BinEncAnySpecial for (u128, u128, u128) {
    #[inline(always)]
    fn any_special(&self) -> bool {
        self.0.wrapping_sub(1) >= MAX_HI
            || self.1.wrapping_sub(1) >= MAX_HI
            || self.2.wrapping_sub(1) >= MAX_HI
    }

    #[inline(always)]
    fn any_subnormal(&self) -> bool {
        self.0 <= HI_FRACTION_MASK
            || self.1 <= HI_FRACTION_MASK
            || self.2 <= HI_FRACTION_MASK
    }

    #[inline(always)]
    fn any_non_normal(&self) -> bool {
        self.any_special() || self.any_subnormal()
    }
}

/// Returns 0 if `abs_bits` represents a subnormal f256 or ZERO, 1 otherwise.
#[inline(always)]
pub(crate) const fn norm_bit(abs_bits: &U256) -> u32 {
    (abs_bits.hi.0 >= HI_FRACTION_BIAS) as u32
}

/// Returns the biased exponent from `abs_bits`.
#[inline(always)]
pub(crate) const fn exp_bits(abs_bits: &U256) -> u32 {
    (abs_bits.hi.0 >> HI_FRACTION_BITS) as u32
}

/// Returns the unbiased exponent from `abs_bits`.
#[inline(always)]
pub(crate) const fn exp(abs_bits: &U256) -> i32 {
    debug_assert!(!abs_bits.is_zero());
    let mut exp = (abs_bits.hi.0 >> HI_FRACTION_BITS) as i32;
    exp + (exp == 0) as i32 - EXP_BIAS as i32
}

/// Returns the fraction from `abs_bits`.
#[inline(always)]
pub(crate) const fn fraction(abs_bits: &U256) -> U256 {
    U256::new(abs_bits.hi.0 & HI_FRACTION_MASK, abs_bits.lo.0)
}

/// Returns the integral significand from `abs_bits`.
#[inline(always)]
pub(crate) const fn signif(abs_bits: &U256) -> U256 {
    U256::new(
        (((abs_bits.hi.0 >= HI_FRACTION_BIAS) as u128) << HI_FRACTION_BITS)
            | (abs_bits.hi.0 & HI_FRACTION_MASK),
        abs_bits.lo.0,
    )
}

/// Returns the normalized integral significand and the corresponding shift
/// from `abs_bits`.
#[inline(always)]
pub(crate) fn norm_signif(abs_bits: &U256) -> (U256, u32) {
    debug_assert!(!abs_bits.is_zero());
    let signif = signif(abs_bits);
    let shift = FRACTION_BITS - signif.msb();
    (signif.shift_left(shift), shift)
}

/// Returns the left adjusted integral significand and the corresponding
/// shift from `abs_bits`.
#[inline(always)]
pub(crate) fn left_adj_signif(abs_bits: &U256) -> (U256, u32) {
    debug_assert!(!abs_bits.is_zero());
    let signif = signif(abs_bits);
    let shift = signif.leading_zeros();
    (signif.shift_left(shift), shift)
}

/// Extract sign, quantum exponent and integral significand from f
#[allow(clippy::cast_possible_wrap)]
pub(crate) const fn split_f256_enc(f: &f256) -> (u32, i32, U256) {
    const TOTAL_BIAS: i32 = EXP_BIAS as i32 + FRACTION_BITS as i32;
    let sign = f.sign();
    let abs_bits = abs_bits(f);
    let exp_bits = exp_bits(&abs_bits);
    let fraction = fraction(&abs_bits);
    match (exp_bits, fraction) {
        (0, U256::ZERO) => (sign, 0, U256::ZERO),
        (0, _) => (sign, 1 - TOTAL_BIAS, fraction),
        (EXP_MAX, _) => (sign, exp_bits as i32, fraction),
        _ => (
            sign,
            exp_bits as i32 - TOTAL_BIAS,
            U256::new(fraction.hi.0 | HI_FRACTION_BIAS, fraction.lo.0),
        ),
    }
}

/// Computes the rounded sum of two f256 values and the remainder.
///
/// Pre-condition: a >= b
#[inline]
pub(crate) fn fast_sum(a: &f256, b: &f256) -> (f256, f256) {
    debug_assert!(a.abs() >= b.abs());
    let s = a + b;
    let r = b - (s - a);
    (s, r)
}

/// Computes the rounded sum of two f256 values and the remainder.
pub(crate) fn sum(a: &f256, b: &f256) -> (f256, f256) {
    let s = a + b;
    let ta = a - (s - b);
    let tb = b - (s - ta);
    let r = ta + tb;
    (s, r)
}

/// Computes the rounded product of two f256 values and the remainder.
#[inline]
pub(crate) fn fast_mul(a: &f256, b: &f256) -> (f256, f256) {
    debug_assert!(a.biased_exponent() + b.biased_exponent() >= EXP_BIAS);
    let p = a * b;
    let r = a.mul_add(*b, -p);
    (p, r)
}

#[cfg(test)]
mod repr_tests {
    use super::*;

    #[test]
    fn test_zero() {
        let z = f256::ZERO;
        assert_eq!(z.sign(), 0);
        assert_eq!(z.quantum_exponent(), 0);
        assert_eq!(z.integral_significand(), U256::default());
        assert_eq!(z.decode(), (0, 0, U256::default()));
        assert_eq!(z.exponent(), 0);
        assert_eq!(z.significand(), f256::ZERO);
        let z = f256::NEG_ZERO;
        assert_eq!(z.sign(), 1);
        assert_eq!(z.quantum_exponent(), 0);
        assert_eq!(z.integral_significand(), U256::default());
        assert_eq!(z.decode(), (1, 0, U256::default()));
        assert_eq!(z.exponent(), 0);
        assert_eq!(z.significand(), f256::ZERO);
    }

    #[test]
    fn test_one() {
        let i = f256::ONE;
        assert_eq!(i.sign(), 0);
        assert_eq!(i.biased_exponent(), EXP_BIAS);
        assert_eq!(i.quantum_exponent(), INT_EXP);
        assert_eq!(
            i.integral_significand(),
            U256::new(1_u128 << HI_FRACTION_BITS, 0)
        );
        assert_eq!(i.decode(), (0, 0, U256::ONE));
        assert_eq!(i.exponent(), 0);
        assert_eq!(i.significand(), f256::ONE);
        let i = f256::NEG_ONE;
        assert_eq!(i.sign(), 1);
        assert_eq!(i.biased_exponent(), EXP_BIAS);
        assert_eq!(i.quantum_exponent(), INT_EXP);
        assert_eq!(
            i.integral_significand(),
            U256::new(1_u128 << HI_FRACTION_BITS, 0)
        );
        assert_eq!(i.decode(), (1, 0, U256::ONE));
        assert_eq!(i.exponent(), 0);
        assert_eq!(i.significand(), f256::ONE);
    }

    #[test]
    fn test_normal() {
        let i = f256::TWO;
        assert_eq!(i.sign(), 0);
        assert_eq!(i.biased_exponent(), EXP_BIAS + 1);
        assert_eq!(i.quantum_exponent(), INT_EXP + 1);
        assert_eq!(
            i.integral_significand(),
            U256::new(1_u128 << HI_FRACTION_BITS, 0)
        );
        assert_eq!(i.decode(), (0, 1, U256::ONE));
        assert_eq!(i.exponent(), 1);
        assert_eq!(i.significand(), f256::ONE);
        let f = f256::from(-3.5_f64);
        assert_eq!(f.sign(), 1);
        assert_eq!(f.quantum_exponent(), -235);
        assert_eq!(
            f.integral_significand(),
            U256::new(567907468902246771870523036008448, 0)
        );
        assert_eq!(f.decode(), (1, -1, U256::new(0_u128, 7_u128)));
        assert_eq!(f.exponent(), 1);
        assert_eq!(f.significand(), f.abs() / f256::TWO);
    }

    #[test]
    #[allow(clippy::cast_possible_wrap)]
    fn test_subnormal() {
        let f = f256::MIN_GT_ZERO;
        assert_eq!(f.sign(), 0);
        assert_eq!(f.quantum_exponent(), EMIN - FRACTION_BITS as i32);
        assert_eq!(f.integral_significand(), U256::ONE);
        assert_eq!(f.decode(), (0, EMIN - FRACTION_BITS as i32, U256::ONE));
        assert_eq!(f.exponent(), EMIN);
        assert_eq!(
            f.significand(),
            f256::from_sign_exp_signif(0, -(FRACTION_BITS as i32), (0, 1))
        );
        let f = f256::from_sign_exp_signif(
            1,
            EMIN - FRACTION_BITS as i32 + 17,
            (7, 29),
        );
        assert!(f.is_subnormal());
        assert_eq!(f.exponent(), EMIN);
        assert_eq!(
            f.significand(),
            f256::from_sign_exp_signif(
                0,
                17 - (FRACTION_BITS as i32),
                (7, 29)
            )
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
        let significand = U256::new(39, 10000730744);
        let f = f256::encode(sign, exponent, significand);
        let (s, t, c) = f.decode();
        let g = f256::encode(s, t, c);
        assert_eq!(f, g);
    }

    #[test]
    fn test_subnormal() {
        let sign = 0_u32;
        let exponent = EMIN - 235_i32;
        let significand = U256::new(u128::MAX >> (EXP_BITS + 2), 0);
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
mod raw_bits_tests {
    use super::*;

    #[test]
    fn test_to_from_bits() {
        let f = f256::TEN;
        let bits = f.to_bits();
        assert_eq!(bits.0, f.bits.hi.0);
        assert_eq!(bits.1, f.bits.lo.0);
        let g = f256::from_bits(bits);
        assert_eq!(f, g);
    }

    #[test]
    fn test_to_from_ne_bytes() {
        let f = f256::TEN;
        let bytes = f.to_ne_bytes();
        let g = f256::from_ne_bytes(bytes);
        assert_eq!(f, g);
    }

    #[test]
    fn test_to_from_be_bytes() {
        let f = f256::TEN;
        let bytes = f.to_be_bytes();
        let g = f256::from_be_bytes(bytes);
        assert_eq!(f, g);
    }

    #[test]
    fn test_to_from_le_bytes() {
        let f = f256::TEN;
        let bytes = f.to_le_bytes();
        let g = f256::from_le_bytes(bytes);
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
        assert_eq!(fi.quantum_exponent(), f.quantum_exponent());
        assert_eq!(ff, f256::ZERO);
        let (gi, gf) = g.split();
        assert_eq!(gi, f);
        assert_eq!(gi.quantum_exponent(), g.quantum_exponent());
        assert_eq!(gf, g - f);
        assert_eq!(h.split(), (-f, (f - g)));
    }

    #[test]
    fn test_lt_1() {
        let f = f256::from(0.99999_f64);
        let (fi, ff) = f.split();
        assert_eq!(fi, f256::ZERO);
        assert_eq!(fi.quantum_exponent(), 0);
        assert_eq!(ff, f);
        assert_eq!(ff.quantum_exponent(), f.quantum_exponent());
    }

    #[test]
    fn test_subnormal() {
        let f = f256::MIN_GT_ZERO;
        assert_eq!(f.split(), (f256::ZERO, f256::MIN_GT_ZERO));
    }
}

#[cfg(test)]
mod ulp_tests {
    use super::*;

    #[test]
    fn test_special() {
        assert_eq!(f256::ZERO.ulp(), MIN_GT_ZERO);
        assert!(f256::INFINITY.ulp().is_nan());
        assert!(f256::NEG_INFINITY.ulp().is_nan());
        assert!(f256::NAN.ulp().is_nan());
    }

    #[test]
    fn test_normal() {
        assert_eq!(f256::ONE.ulp(), f256::EPSILON);
        assert_eq!(f256::TWO.ulp(), f256::TWO * f256::EPSILON);
        assert_eq!(f256::from(3).ulp(), f256::TWO * f256::EPSILON);
        assert_eq!(f256::TEN.ulp(), f256::from(8) * f256::EPSILON);
        assert_eq!(f256::MIN_POSITIVE.ulp(), f256::MIN_GT_ZERO);
        assert_eq!(
            f256::MIN.ulp(),
            f256::from_sign_exp_signif(
                0,
                EMAX - FRACTION_BITS as i32,
                (0, 1),
            )
        );
    }

    #[test]
    fn test_subnormal() {
        let f = f256::MIN_POSITIVE - f256::MIN_GT_ZERO;
        assert_eq!(f.ulp(), f256::MIN_GT_ZERO);
        assert_eq!(f256::MIN_GT_ZERO.ulp(), f256::MIN_GT_ZERO);
    }
}

#[cfg(test)]
mod mul_pow2_tests {
    use super::*;

    #[test]
    fn test_special() {
        assert_eq!(f256::ZERO.mul_pow2(4), f256::ZERO);
        assert_eq!(f256::INFINITY.mul_pow2(1), f256::INFINITY);
        assert_eq!(f256::NEG_INFINITY.mul_pow2(1), f256::NEG_INFINITY);
        assert!(f256::NAN.mul_pow2(38).is_nan());
    }

    #[test]
    fn test_normal() {
        let f = f256::TEN.mul_pow2(4);
        assert_eq!(f, f256::from(160));
        let g = f256::from(0.0793).mul_pow2(3);
        assert_eq!(g, f256::from(0.6344));
    }

    #[test]
    fn test_overflow() {
        let f = f256::MAX.mul_pow2(1);
        assert_eq!(f, f256::INFINITY);
        let f = f256::MIN.mul_pow2(5);
        assert_eq!(f, f256::NEG_INFINITY);
    }

    #[test]
    fn test_subnormal() {
        let f = f256::MIN_POSITIVE - f256::MIN_GT_ZERO;
        assert!(f.is_subnormal());
        let g = f.mul_pow2(1);
        assert!(g.is_normal());
        assert_eq!(g, f * f256::TWO);
        let f = f256::MIN_GT_ZERO;
        let g = f.mul_pow2(5);
        assert!(g.is_subnormal());
        assert_eq!(g, f * f256::from(32));
    }

    #[test]
    fn test_subnormal_overflow() {
        let f = f256::MIN_GT_ZERO;
        let g = f.mul_pow2(u32::MAX);
        assert_eq!(g, f256::INFINITY);
    }
}

#[cfg(test)]
mod div_pow2_tests {
    use super::*;

    #[test]
    fn test_special() {
        assert_eq!(f256::ZERO.div_pow2(4), f256::ZERO);
        assert_eq!(f256::INFINITY.div_pow2(1), f256::INFINITY);
        assert_eq!(f256::NEG_INFINITY.div_pow2(1), f256::NEG_INFINITY);
        assert!(f256::NAN.div_pow2(38).is_nan());
    }

    #[test]
    fn test_normal() {
        let f = f256::TEN;
        assert_eq!(f.div_pow2(3), f256::from(1.25));
        let g = f256::from(0.0793);
        assert_eq!(g.div_pow2(5), g / f256::from(32));
        let h = f256::MIN_POSITIVE.negated();
        assert_eq!(h.div_pow2(65), h / f256::from(36893488147419103232_u128));
        let f = f256::MIN_POSITIVE.div_pow2(236);
        assert_eq!(f, f256::MIN_GT_ZERO);
    }

    #[test]
    fn test_underflow() {
        let f = f256::MIN_GT_ZERO.div_pow2(1);
        assert_eq!(f, f256::ZERO);
        let f = f256::MIN_POSITIVE.negated().div_pow2(SIGNIFICAND_BITS);
        assert_eq!(f, f256::NEG_ZERO);
    }

    #[test]
    fn test_subnormal() {
        let f = f256::MIN_POSITIVE - f256::MIN_GT_ZERO;
        assert!(f.is_subnormal());
        let g = f.div_pow2(7);
        assert!(g.is_subnormal());
        assert_eq!(g, f / f256::from(128));
    }
}
