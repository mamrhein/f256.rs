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

use core::{cmp::Ordering, num::FpCategory};

use crate::float256::{Float256Repr, EMAX, EMIN, SIGNIFICAND_BITS};

mod binops;
mod float256;
mod from_float;
mod uint256;

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
    /// 1 - MAX_EXP = -262142.
    pub const MIN_EXP: i32 = EMIN;

    /// Maximum possible power of 10 exponent: ⌊log₁₀(2)⌋.
    pub const MAX_10_EXP: i32 = 78912;

    /// Minimum possible normal power of 10 exponent ( .
    pub const MIN_10_EXP: i32 = -78911;

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
    // TODO: replace by -ZERO
    pub const NEG_ZERO: Self = Self {
        repr: Float256Repr::NEG_ZERO,
    };

    /// Multiplicative identity
    pub const ONE: Self = Self {
        repr: Float256Repr::ONE,
    };

    /// Multiplicative negator
    // TODO: replace by -ONE
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
    // TODO: uncomment when Div implemented
    // #[must_use]
    // #[inline]
    // pub fn recip(self) -> Self {
    //     Self::ONE / self
    // }

    /// Converts radians to degrees.
    // #[must_use = "this returns the result of the operation, \
    //               without modifying the original"]
    // #[inline]
    // pub fn to_degrees(self) -> Self {
    // The division here is correctly rounded with respect to the true
    // value of 180/π. (This differs from f32, where a constant must be
    // used to ensure a correctly rounded result.)
    //     self * (180.0f256 / consts::PI)
    // }

    /// Converts degrees to radians.
    // #[must_use = "this returns the result of the operation, \
    //               without modifying the original"]
    // #[inline]
    // pub fn to_radians(self) -> Self {
    //     let value: Self = consts::PI;
    //     self * (value / 180.0)
    // }

    /// Returns the maximum of the two numbers, ignoring NaN.
    ///
    /// If one of the arguments is NaN, then the other argument is returned.
    /// This follows the IEEE-754 2008 semantics for maxNum, except for handling
    /// of signaling NaNs; this function handles all NaNs the same way and
    /// avoids maxNum's problems with associativity. This also matches the
    /// behavior of libm’s fmax.
    #[must_use]
    #[inline]
    pub fn max(self, other: Self) -> Self {
        unimplemented!()
    }

    /// Returns the minimum of the two numbers, ignoring NaN.
    ///
    /// If one of the arguments is NaN, then the other argument is returned.
    /// This follows the IEEE-754 2008 semantics for minNum, except for handling
    /// of signaling NaNs; this function handles all NaNs the same way and
    /// avoids minNum's problems with associativity. This also matches the
    /// behavior of libm’s fmin.
    #[must_use]
    #[inline]
    pub fn min(self, other: Self) -> Self {
        unimplemented!()
    }

    /// Returns the maximum of the two numbers, propagating NaN.
    ///
    /// This returns NaN when *either* argument is NaN, as opposed to
    /// [`f256::max`] which only returns NaN when *both* arguments are NaN.
    ///
    /// If one of the arguments is NaN, then NaN is returned. Otherwise this
    /// returns the greater of the two numbers. For this operation, -0.0 is
    /// considered to be less than +0.0. Note that this follows the
    /// semantics specified in IEEE 754-2019.
    ///
    /// Also note that "propagation" of NaNs here doesn't necessarily mean that
    /// the bitpattern of a NaN operand is conserved; see [explanation of
    /// NaN as a special value](f32) for more info.
    #[must_use]
    #[inline]
    pub fn maximum(self, other: Self) -> Self {
        // if self > other {
        //     self
        // } else if other > self {
        //     other
        // } else if self == other {
        //     if self.is_sign_positive() && other.is_sign_negative() {
        //         self
        //     } else {
        //         other
        //     }
        // } else {
        //     self + other
        // }
        unimplemented!()
    }

    /// Returns the minimum of the two numbers, propagating NaN.
    ///
    /// This returns NaN when *either* argument is NaN, as opposed to
    /// [`f256::min`] which only returns NaN when *both* arguments are NaN.
    ///
    /// If one of the arguments is NaN, then NaN is returned. Otherwise this
    /// returns the lesser of the two numbers. For this operation, -0.0 is
    /// considered to be less than +0.0. Note that this follows the
    /// semantics specified in IEEE 754-2019.
    ///
    /// Also note that "propagation" of NaNs here doesn't necessarily mean that
    /// the bitpattern of a NaN operand is conserved; see [explanation of
    /// NaN as a special value](f32) for more info.
    #[must_use]
    #[inline]
    pub fn minimum(self, other: Self) -> Self {
        // if self < other {
        //     self
        // } else if other < self {
        //     other
        // } else if self == other {
        //     if self.is_sign_negative() && other.is_sign_positive() {
        //         self
        //     } else {
        //         other
        //     }
        // } else {
        //     self + other
        // }
        unimplemented!()
    }

    /// Raw transmutation to `u64`.
    ///
    /// This is currently identical to `transmute::<f256, u64>(self)` on all
    /// platforms.
    ///
    /// See [`from_bits`](Self::from_bits) for some discussion of the
    /// portability of this operation (there are almost no issues).
    ///
    /// Note that this function is distinct from `as` casting, which attempts to
    /// preserve the *numeric* value, and not the bitwise value.
    // #[must_use = "this returns the result of the operation, \
    //               without modifying the original"]
    // #[stable(feature = "float_bits_conv", since = "1.20.0")]
    // #[rustc_const_unstable(feature = "const_float_bits_conv", issue =
    // "72447")] #[inline]
    // pub const fn to_bits(self) -> u64 {
    //     // SAFETY: `u64` is a plain old datatype so we can always transmute
    // to it.     // ...sorta.
    //     //
    //     // See the SAFETY comment in f256::from_bits for more.
    //     #[rustc_const_unstable(feature = "const_float_bits_conv", issue =
    // "72447")]     const fn ct_f256_to_u64(ct: Self) -> u64 {
    //         match ct.classify() {
    //             FpCategory::Nan => {
    //                 panic!("const-eval error: cannot use f256::to_bits on a
    // NaN")             }
    //             FpCategory::Subnormal => {
    //                 panic!("const-eval error: cannot use f256::to_bits on a
    // subnormal number")             }
    //             FpCategory::Infinite | FpCategory::Normal | FpCategory::Zero
    // => {                 // SAFETY: We have a normal floating point
    // number. Now we transmute, i.e. do a bitcopy.                 unsafe {
    // mem::transmute::<f256, u64>(ct) }             }
    //         }
    //     }
    //     // SAFETY: `u64` is a plain old datatype so we can always... uh...
    //     // ...look, just pretend you forgot what you just read.
    //     // Stability concerns.
    //     let rt_f256_to_u64 = |rt| unsafe { mem::transmute::<f256, u64>(rt) };
    //     // SAFETY: We use internal implementations that either always work or
    // fail at compile time.     unsafe {
    // intrinsics::const_eval_select((self,), ct_f256_to_u64, rt_f256_to_u64) }
    // }

    /// Raw transmutation to `[u64; 4]`.
    #[inline]
    const fn to_bits(&self) -> [u64; 4] {
        // (self.bits.hi, self.bits.lo)
        unimplemented!()
    }

    /// Raw transmutation from `u64`.
    ///
    /// This is currently identical to `transmute::<u64, f256>(v)` on all
    /// platforms. It turns out this is incredibly portable, for two
    /// reasons:
    ///
    /// * Floats and Ints have the same endianness on all supported platforms.
    /// * IEEE-754 very precisely specifies the bit layout of floats.
    ///
    /// However there is one caveat: prior to the 2008 version of IEEE-754, how
    /// to interpret the NaN signaling bit wasn't actually specified. Most
    /// platforms (notably x86 and ARM) picked the interpretation that was
    /// ultimately standardized in 2008, but some didn't (notably MIPS). As
    /// a result, all signaling NaNs on MIPS are quiet NaNs on x86, and
    /// vice-versa.
    ///
    /// Rather than trying to preserve signaling-ness cross-platform, this
    /// implementation favors preserving the exact bits. This means that
    /// any payloads encoded in NaNs will be preserved even if the result of
    /// this method is sent over the network from an x86 machine to a MIPS one.
    ///
    /// If the results of this method are only manipulated by the same
    /// architecture that produced them, then there is no portability concern.
    ///
    /// If the input isn't NaN, then there is no portability concern.
    ///
    /// If you don't care about signaling-ness (very likely), then there is no
    /// portability concern.
    ///
    /// Note that this function is distinct from `as` casting, which attempts to
    /// preserve the *numeric* value, and not the bitwise value.
    // #[must_use]
    // #[inline]
    // pub const fn from_bits(v: u64) -> Self {
    // It turns out the safety issues with sNaN were overblown! Hooray!
    // SAFETY: `u64` is a plain old datatype so we can always transmute from it
    // ...sorta.
    //
    // It turns out that at runtime, it is possible for a floating point number
    // to be subject to floating point modes that alter nonzero subnormal
    // numbers to zero on reads and writes, aka "denormals are zero" and
    // "flush to zero". This is not a problem usually, but at least one
    // tier2 platform for Rust actually exhibits an FTZ behavior by default:
    // thumbv7neon aka "the Neon FPU in AArch32 state"
    //
    // Even with this, not all instructions exhibit the FTZ behaviors on
    // thumbv7neon, so this should load the same bits if LLVM emits the
    // "correct" instructions, but LLVM sometimes makes interesting choices
    // about float optimization, and other FPUs may do similar. Thus, it is
    // wise to indulge luxuriously in caution.
    //
    // In addition, on x86 targets with SSE or SSE2 disabled and the x87 FPU
    // enabled, i.e. not soft-float, the way Rust does parameter passing can
    // actually alter a number that is "not infinity" to have the same
    // exponent as infinity, in a slightly unpredictable manner.
    //
    // And, of course evaluating to a NaN value is fairly nondeterministic.
    // More precisely: when NaN should be returned is knowable, but which NaN?
    // So far that's defined by a combination of LLVM and the CPU, not Rust.
    // This function, however, allows observing the bitstring of a NaN,
    // thus introspection on CTFE.
    //
    // In order to preserve, at least for the moment, const-to-runtime
    // equivalence, reject any of these possible situations from happening.
    // #[rustc_const_unstable(feature = "const_float_bits_conv", issue =
    // "72447")]     const fn ct_u64_to_f256(ct: u64) -> Self {
    //         match f256::classify_bits(ct) {
    //             FpCategory::Subnormal => {
    //                 panic!("const-eval error: cannot use f256::from_bits on a
    // subnormal number")             }
    //             FpCategory::Nan => {
    //                 panic!("const-eval error: cannot use f256::from_bits on
    // NaN")             }
    //             FpCategory::Infinite | FpCategory::Normal | FpCategory::Zero
    // => {                 // SAFETY: It's not a frumious number
    //                 unsafe { mem::transmute::<u64, f256>(ct) }
    //             }
    //         }
    //     }
    //     // SAFETY: `u64` is a plain old datatype so we can always... uh...
    //     // ...look, just pretend you forgot what you just read.
    //     // Stability concerns.
    //     let rt_u64_to_f256 = |rt| unsafe { mem::transmute::<u64, f256>(rt) };
    //     // SAFETY: We use internal implementations that either always work or
    // fail at compile time.     unsafe {
    // intrinsics::const_eval_select((v,), ct_u64_to_f256, rt_u64_to_f256) }
    // }

    /// Raw transmutation from `[u64; 4]`.
    #[inline]
    const fn from_bits(hi: u128, lo: u128) -> Self {
        // Self {
        //     repr: u256 { hi, lo },
        // }
        unimplemented!()
    }

    /// Return the memory representation of this floating point number as a byte
    /// array in big-endian (network) byte order.
    ///
    /// See [`from_bits`](Self::from_bits) for some discussion of the
    /// portability of this operation (there are almost no issues).
    #[must_use]
    #[inline]
    pub const fn to_be_bytes(self) -> [u8; 8] {
        // self.to_bits().to_be_bytes()'
        unimplemented!()
    }

    /// Return the memory representation of this floating point number as a byte
    /// array in little-endian byte order.
    ///
    /// See [`from_bits`](Self::from_bits) for some discussion of the
    /// portability of this operation (there are almost no issues).
    #[must_use]
    #[inline]
    pub const fn to_le_bytes(self) -> [u8; 8] {
        // self.to_bits().to_le_bytes()
        unimplemented!()
    }

    /// Return the memory representation of this floating point number as a byte
    /// array in native byte order.
    ///
    /// As the target platform's native endianness is used, portable code
    /// should use [`to_be_bytes`] or [`to_le_bytes`], as appropriate, instead.
    ///
    /// [`to_be_bytes`]: Self::to_be_bytes
    /// [`to_le_bytes`]: Self::to_le_bytes
    ///
    /// See [`from_bits`](Self::from_bits) for some discussion of the
    /// portability of this operation (there are almost no issues).
    #[must_use]
    #[inline]
    pub const fn to_ne_bytes(self) -> [u8; 8] {
        // self.to_bits().to_ne_bytes()
        unimplemented!()
    }

    /// Create a floating point value from its representation as a byte array in
    /// big endian.
    ///
    /// See [`from_bits`](Self::from_bits) for some discussion of the
    /// portability of this operation (there are almost no issues).
    ///
    /// # Examples
    ///
    /// ```
    /// let value =
    ///     f256::from_be_bytes([0x40, 0x29, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
    /// assert_eq!(value, 12.5);
    /// ```
    #[must_use]
    #[inline]
    pub const fn from_be_bytes(bytes: [u8; 8]) -> Self {
        unimplemented!()
    }

    /// Create a floating point value from its representation as a byte array in
    /// little endian.
    ///
    /// See [`from_bits`](Self::from_bits) for some discussion of the
    /// portability of this operation (there are almost no issues).
    ///
    /// # Examples
    ///
    /// ```
    /// let value =
    ///     f256::from_le_bytes([0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x29, 0x40]);
    /// assert_eq!(value, 12.5);
    /// ```
    #[must_use]
    #[inline]
    pub const fn from_le_bytes(bytes: [u8; 8]) -> Self {
        unimplemented!()
    }

    /// Create a floating point value from its representation as a byte array in
    /// native endian.
    ///
    /// As the target platform's native endianness is used, portable code
    /// likely wants to use [`from_be_bytes`] or [`from_le_bytes`], as
    /// appropriate instead.
    ///
    /// [`from_be_bytes`]: Self::from_be_bytes
    /// [`from_le_bytes`]: Self::from_le_bytes
    ///
    /// See [`from_bits`](Self::from_bits) for some discussion of the
    /// portability of this operation (there are almost no issues).
    ///
    /// # Examples
    ///
    /// ```
    /// let value = f256::from_ne_bytes(if cfg!(target_endian = "big") {
    ///     [0x40, 0x29, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
    /// } else {
    ///     [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x29, 0x40]
    /// });
    /// assert_eq!(value, 12.5);
    /// ```
    #[must_use]
    #[inline]
    pub const fn from_ne_bytes(bytes: [u8; 8]) -> Self {
        unimplemented!()
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
        assert!(!f256::INFINITY.is_nan());
        assert!(!f256::NEG_INFINITY.is_nan());
    }

    #[test]
    fn test_inf() {
        assert!(f256::INFINITY.is_infinite());
        assert!(f256::NEG_INFINITY.is_infinite());
        assert!(!f256::INFINITY.is_finite());
        assert!(!f256::NEG_INFINITY.is_finite());
    }
}
