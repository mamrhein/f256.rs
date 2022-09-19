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

use core::{cmp::Ordering, num::FpCategory};

use crate::float256::{Float256Repr, EMAX, EMIN, SIGNIFICAND_BITS};

mod binops;
mod float256;
mod from_float;
mod uint256;
mod unops;

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug)]
pub struct f256 {
    pub(crate) repr: Float256Repr,
}

impl f256 {
    /// The radix or base of the internal representation of `f256`.
    pub const RADIX: u32 = 2;

    /// Number of significant digits in base 2: 237.
    pub const MANTISSA_DIGITS: u32 = SIGNIFICAND_BITS;

    /// Number of significant digits in base 2: 237.
    pub const SIGNIFICANT_DIGITS: u32 = SIGNIFICAND_BITS;

    /// Approximate number of significant digits in base 10 (.
    pub const DIGITS: u32 = 71;

    /// The difference between `1.0` and the next larger representable number:
    /// 2⁻²³⁶ ≈ 9.055679e-72.
    pub const EPSILON: Self = Self {
        repr: Float256Repr::EPSILON,
    };

    /// Largest finite `f256` value:  2²⁶²¹⁴⁴ − 2²⁶¹⁹⁰⁷ ≈ 1.6113e78913.
    pub const MAX: Self = Self {
        repr: Float256Repr::MAX,
    };

    /// Smallest finite `f256` value: 2²⁶¹⁹⁰⁷ - 2²⁶²¹⁴⁴ ≈ -1.6113e78913.
    pub const MIN: Self = Self {
        repr: Float256Repr::MIN,
    };

    /// Smallest positive normal `f256` value: 2⁻²⁶²¹⁴² ≈ 2.4824e−78913.
    pub const MIN_POSITIVE: Self = Self {
        repr: Float256Repr::MIN_POSITIVE,
    };

    /// Maximum possible power of 2 exponent: 2¹⁸ = 262144.
    pub const MAX_EXP: i32 = EMAX + 1;

    /// One greater than the minimum possible normal power of 2 exponent:
    /// 3 - MAX_EXP = -262141.
    pub const MIN_EXP: i32 = EMIN + 1;

    /// Maximum possible power of 10 exponent: ⌊MAX_EXP × log₁₀(2)⌋.
    pub const MAX_10_EXP: i32 = 78913;

    /// Minimum possible normal power of 10 exponent ⌊MIN_EXP × log₁₀(2)⌋.
    pub const MIN_10_EXP: i32 = -78912;

    /// Not a Number (NaN).
    ///
    /// Note that IEEE-745 doesn't define just a single NaN value; a plethora of
    /// bit patterns are considered to be NaN. Furthermore, the standard makes a
    /// difference between a "signaling" and a "quiet" NaN, and allows
    /// inspecting its "payload" (the unspecified bits in the bit pattern).
    /// This implementation does not make such a difference and uses exactly one
    /// bit pattern for NaN.
    pub const NAN: Self = Self {
        repr: Float256Repr::NAN,
    };

    /// Infinity (∞).
    pub const INFINITY: Self = Self {
        repr: Float256Repr::INFINITY,
    };

    /// Negative infinity (−∞).
    pub const NEG_INFINITY: Self = Self {
        repr: Float256Repr::NEG_INFINITY,
    };

    /// Additive identity
    pub const ZERO: Self = Self {
        repr: Float256Repr::ZERO,
    };

    /// Negative additive identity
    pub const NEG_ZERO: Self = Self {
        repr: Float256Repr::NEG_ZERO,
    };

    /// Multiplicative identity
    pub const ONE: Self = Self {
        repr: Float256Repr::ONE,
    };

    /// Multiplicative negator
    pub const NEG_ONE: Self = Self {
        repr: Float256Repr::NEG_ONE,
    };

    /// Equivalent of 2.0: 2 × ONE.
    pub const TWO: Self = Self {
        repr: Float256Repr::TWO,
    };

    /// TODO: Equivalent of 10.0
    // pub const TEN: Self = Self {
    //     bits: u256 {
    //         hi: ((1 + EXP_BIAS) as u128) << HI_FRACTION_BITS,
    //         lo: 0,
    //     },
    // };

    /// Returns `true` if this value is `NaN`.
    #[must_use]
    #[inline]
    pub const fn is_nan(self) -> bool {
        self.repr.is_nan()
    }

    /// Returns `true` if this value is positive infinity or negative infinity,
    /// and `false` otherwise.
    #[must_use]
    #[inline]
    pub const fn is_infinite(self) -> bool {
        self.repr.is_infinite()
    }

    /// Returns `true` if this number is neither infinite nor NaN.
    #[must_use]
    #[inline]
    pub const fn is_finite(self) -> bool {
        self.repr.is_finite()
    }

    /// Returns `true` if the number is subnormal.
    #[must_use]
    #[inline]
    pub const fn is_subnormal(self) -> bool {
        self.repr.is_subnormal()
    }

    /// Returns `true` if the number is neither zero, infinite,
    /// subnormal, or NaN.
    #[must_use]
    #[inline]
    pub const fn is_normal(self) -> bool {
        self.repr.is_normal()
    }

    /// Returns the floating point category of the number. If only one property
    /// is going to be tested, it is generally faster to use the specific
    /// predicate instead.
    #[inline]
    pub const fn classify(&self) -> FpCategory {
        self.repr.classify()
    }

    /// Returns `true` if `self` is equal to `+0.0` or `-0.0`.
    #[must_use]
    #[inline]
    pub const fn is_zero(self) -> bool {
        self.repr.is_zero()
    }

    /// Returns `true` if `self` is either not a number, infinite or equal to
    /// zero.
    #[must_use]
    #[inline]
    pub(crate) const fn is_special(self) -> bool {
        self.repr.is_special()
    }

    /// Returns `true` if `self` has a positive sign, including `+0.0`, positive
    /// infinity and NaN.
    #[must_use]
    #[inline]
    pub const fn is_sign_positive(self) -> bool {
        self.repr.is_sign_positive()
    }

    /// Returns `true` if `self` has a negative sign, including `-0.0`and
    /// negative infinity.
    #[must_use]
    #[inline]
    pub const fn is_sign_negative(self) -> bool {
        self.repr.is_sign_negative()
    }

    /// Returns the reciprocal (inverse) of `self`.
    #[must_use]
    #[inline]
    pub fn recip(self) -> Self {
        // TODO: uncomment when Div implemented
        // Self::ONE / self
        unimplemented!()
    }

    /// Converts radians to degrees.
    #[must_use]
    #[inline]
    pub fn to_degrees(self) -> Self {
        // self * (180.0f256 / consts::PI)
        unimplemented!()
    }

    /// Converts degrees to radians.
    #[must_use]
    #[inline]
    pub fn to_radians(self) -> Self {
        // let value: Self = consts::PI;
        // self * (value / 180.0)
        unimplemented!()
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
    pub const fn to_bits(&self) -> [u64; 4] {
        self.repr.to_bits()
    }

    /// Raw transmutation from `[u64; 4]` (in native endian order).
    #[inline]
    pub const fn from_bits(bits: [u64; 4]) -> Self {
        Self {
            repr: Float256Repr::from_bits(bits),
        }
    }

    /// Return the memory representation of this floating point number as a byte
    /// array in big-endian (network) byte order.
    #[must_use]
    #[inline]
    pub const fn to_be_bytes(self) -> [u8; 32] {
        self.repr.to_be_bytes()
    }

    /// Return the memory representation of this floating point number as a byte
    /// array in little-endian byte order.
    #[must_use]
    #[inline]
    pub const fn to_le_bytes(self) -> [u8; 32] {
        self.repr.to_le_bytes()
    }

    /// Return the memory representation of this floating point number as a byte
    /// array in native byte order.
    #[must_use]
    #[inline]
    pub const fn to_ne_bytes(self) -> [u8; 32] {
        self.repr.to_ne_bytes()
    }

    /// Create a floating point value from its representation as a byte array in
    /// big endian.
    #[must_use]
    #[inline]
    pub const fn from_be_bytes(bytes: [u8; 32]) -> Self {
        Self {
            repr: Float256Repr::from_be_bytes(bytes),
        }
    }

    /// Create a floating point value from its representation as a byte array in
    /// little endian.
    #[must_use]
    #[inline]
    pub const fn from_le_bytes(bytes: [u8; 32]) -> Self {
        Self {
            repr: Float256Repr::from_le_bytes(bytes),
        }
    }

    /// Create a floating point value from its representation as a byte array in
    /// native endian.
    #[must_use]
    #[inline]
    pub const fn from_ne_bytes(bytes: [u8; 32]) -> Self {
        Self {
            repr: Float256Repr::from_ne_bytes(bytes),
        }
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
        self.repr.cmp(&(*other).repr)
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
    /// ```
    #[must_use]
    #[inline]
    pub fn clamp(self, min: Self, max: Self) -> Self {
        // assert!(min <= max);
        // let mut x = self;
        // if x < min {
        //     x = min;
        // }
        // if x > max {
        //     x = max;
        // }
        // x
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nan() {
        assert!(f256::NAN.is_nan());
        assert!((-f256::NAN).is_nan());
        assert!(!f256::INFINITY.is_nan());
        assert!(!f256::NEG_INFINITY.is_nan());
        assert!(!f256::ZERO.is_nan());
        assert!(!f256::NEG_ZERO.is_nan());
    }

    #[test]
    fn test_inf() {
        assert!(f256::INFINITY.is_infinite());
        assert!(f256::NEG_INFINITY.is_infinite());
        assert!(!f256::NAN.is_infinite());
        assert!(!f256::NEG_ZERO.is_infinite());
        assert!(!f256::ZERO.is_infinite());
        assert!(!f256::ONE.is_infinite());
        assert!(!f256::INFINITY.is_finite());
        assert!(!f256::NEG_INFINITY.is_finite());
        assert!(!f256::NAN.is_finite());
        assert!(f256::NEG_ZERO.is_finite());
        assert!(f256::ZERO.is_finite());
        assert!(f256::ONE.is_finite());
    }
}
