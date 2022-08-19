// ---------------------------------------------------------------------------
// Copyright:   (c) 2022 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use crate::rawfloat::RawFloat;
use crate::u256::u256;
use std::num::FpCategory;

const PREC_LEVEL: u32 = 8;
const TOTAL_BITS: u32 = 1_u32 << PREC_LEVEL;
const EXP_BITS: u32 = 4 * PREC_LEVEL - 13;
const EXP_MAX: u32 = (1_u32 << EXP_BITS) - 1;
const EXP_BIAS: u32 = EXP_MAX >> 1;
const EMAX: i32 = EXP_BIAS as i32;
const EMIN: i32 = 1 - EMAX;
const FRACTION_BITS: u32 = TOTAL_BITS - EXP_BITS - 1;
const HI_TOTAL_BITS: u32 = TOTAL_BITS >> 1;
const HI_SIGN_SHIFT: u32 = HI_TOTAL_BITS - 1;
const HI_FRACTION_BITS: u32 = FRACTION_BITS - HI_TOTAL_BITS;
const HI_FRACTION_BIAS: u128 = 1_u128 << HI_FRACTION_BITS;
const HI_FRACTION_MASK: u128 = HI_FRACTION_BIAS - 1;
const HI_EXP_MASK: u128 = (EXP_MAX as u128) << HI_FRACTION_BITS;
const HI_SIGN_MASK: u128 = 1_u128 << 127;
const NAN_HI: u128 = HI_EXP_MASK + 1;
const INF_HI: u128 = HI_EXP_MASK;
const NEG_INF_HI: u128 = HI_SIGN_MASK | HI_EXP_MASK;

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug)]
pub struct f256 {
    bits: u256,
}

impl f256 {
    /// The radix or base of the internal representation of `f256`.
    pub const RADIX: u32 = 2;

    /// Number of significant digits in base 2.
    pub const MANTISSA_DIGITS: u32 = FRACTION_BITS + 1;

    /// Number of significant digits in base 2.
    pub const SIGNIFICANT_DIGITS: u32 = FRACTION_BITS + 1;

    /// Approximate number of significant digits in base 10.
    pub const DIGITS: u32 = 71;

    /// This is the difference between `1.0` and the next larger representable
    /// number.
    // pub const EPSILON: f256 = TODO: 2.2204460492503131e-16_f256;

    /// Smallest finite `f256` value.
    // pub const MIN: f256 = TODO: -1.7976931348623157e+308_f256;

    /// Smallest positive normal `f256` value.
    // pub const MIN_POSITIVE: f256 = TODO: 2.2250738585072014e-308_f256;

    /// Largest finite `f256` value.
    // pub const MAX: f256 = TODO: 1.7976931348623157e+308_f256;

    /// One greater than the minimum possible normal power of 2 exponent.
    pub const MIN_EXP: i32 = EMIN;

    /// Maximum possible power of 2 exponent.
    pub const MAX_EXP: i32 = EMAX;

    /// Minimum possible normal power of 10 exponent.
    pub const MIN_10_EXP: i32 = -78911;

    /// Maximum possible power of 10 exponent.
    pub const MAX_10_EXP: i32 = 78912;

    /// Not a Number (NaN).
    ///
    /// Note that IEEE-745 doesn't define just a single NaN value; a plethora of
    /// bit patterns are considered to be NaN. Furthermore, the standard makes a
    /// difference between a "signaling" and a "quiet" NaN, and allows
    /// inspecting its "payload" (the unspecified bits in the bit pattern).
    /// This constant isn't guaranteed to equal to any specific NaN bitpattern,
    /// and the stability of its representation over Rust versions and target
    /// platforms isn't guaranteed.
    pub const NAN: f256 = f256::from_bits(NAN_HI, 0);

    /// Infinity (∞).
    pub const INFINITY: f256 = f256::from_bits(INF_HI, 0);

    /// Negative infinity (−∞).
    pub const NEG_INFINITY: f256 = f256::from_bits(NEG_INF_HI, 0);

    /// Raw transmutation from `(u128, u128)`.
    #[inline]
    const fn from_bits(hi: u128, lo: u128) -> Self {
        Self {
            bits: u256 { hi, lo },
        }
    }

    /// Raw transmutation to `(u128, u128)`.
    #[inline]
    const fn to_bits(&self) -> (u128, u128) {
        (*self.bits.hi, *self.bits.lo)
    }

    /// Returns the sign bit of `self`: 0 = positive, 1 = negative.
    #[inline]
    fn sign(&self) -> u32 {
        (self.bits.hi >> HI_SIGN_SHIFT) as u32
    }

    /// Returns `true` if this value is NaN.
    #[must_use]
    #[inline]
    pub const fn is_nan(self) -> bool {
        self.bits.hi == NAN_HI
    }
    /// Returns `true` if this value is positive infinity or negative infinity, and
    /// `false` otherwise.
    #[must_use]
    #[inline]
    pub const fn is_infinite(self) -> bool {
        (self.bits.hi & (HI_EXP_MASK ^ 0)) == HI_EXP_MASK
    }

    /// Returns `true` if this number is neither infinite nor NaN.
    #[must_use]
    #[inline]
    pub const fn is_finite(self) -> bool {
        (self.bits.hi & HI_EXP_MASK) != HI_EXP_MASK
    }

    /// Returns `true` if the number is [subnormal].
    #[must_use]
    #[inline]
    pub const fn is_subnormal(self) -> bool {
        matches!(self.classify(), FpCategory::Subnormal)
    }

    /// Returns `true` if the number is neither zero, infinite,
    /// [subnormal], or NaN.
    #[must_use]
    #[inline]
    pub const fn is_normal(self) -> bool {
        matches!(self.classify(), FpCategory::Normal)
    }

    /// Returns the floating point category of the number. If only one property
    /// is going to be tested, it is generally faster to use the specific
    /// predicate instead.
    pub const fn classify(self) -> FpCategory {
        // A previous implementation tried to only use bitmask-based checks,
        // using f256::to_bits to transmute the float to its bit repr and match on that.
        // Unfortunately, floating point numbers can be much worse than that.
        // This also needs to not result in recursive evaluations of f256::to_bits.
        //
        // On some processors, in some cases, LLVM will "helpfully" lower floating point ops,
        // in spite of a request for them using f32 and f256, to things like x87 operations.
        // These have an f256's mantissa, but can have a larger than normal exponent.
        // FIXME(jubilee): Using x87 operations is never necessary in order to function
        // on x86 processors for Rust-to-Rust calls, so this issue should not happen.
        // Code generation should be adjusted to use non-C calling conventions, avoiding this.
        //
        // Thus, a value may compare unequal to infinity, despite having a "full" exponent mask.
        // And it may not be NaN, as it can simply be an "overextended" finite value.
        if self.is_nan() {
            FpCategory::Nan
        } else {
            // However, std can't simply compare to zero to check for zero, either,
            // as correctness requires avoiding equality tests that may be Subnormal == -0.0
            // because it may be wrong under "denormals are zero" and "flush to zero" modes.
            // Most of std's targets don't use those, but they are used for thumbv7neon.
            // So, this does use bitpattern matching for the rest.

            // SAFETY: f256 to u64 is fine. Usually.
            // If control flow has gotten this far, the value is definitely in one of the categories
            // that f256::partial_classify can correctly analyze.
            unsafe { f256::partial_classify(self) }
        }
    }

    const fn classify_bits(b: u64) -> FpCategory {
        const EXP_MASK: u64 = 0x7ff0000000000000;
        const MAN_MASK: u64 = 0x000fffffffffffff;

        match (b & MAN_MASK, b & EXP_MASK) {
            (0, EXP_MASK) => FpCategory::Infinite,
            (_, EXP_MASK) => FpCategory::Nan,
            (0, 0) => FpCategory::Zero,
            (_, 0) => FpCategory::Subnormal,
            _ => FpCategory::Normal,
        }
    }

    /// Returns `true` if `self` has a positive sign, including `+0.0`, NaNs with
    /// positive sign bit and positive infinity. Note that IEEE-745 doesn't assign any
    /// meaning to the sign bit in case of a NaN, and as Rust doesn't guarantee that
    /// the bit pattern of NaNs are conserved over arithmetic operations, the result of
    /// `is_sign_positive` on a NaN might produce an unexpected result in some cases.
    /// See [explanation of NaN as a special value](f32) for more info.
    #[must_use]
    #[inline]
    pub const fn is_sign_positive(self) -> bool {
        !self.is_sign_negative()
    }

    #[must_use]
    #[inline]
    #[doc(hidden)]
    pub fn is_positive(self) -> bool {
        self.is_sign_positive()
    }

    /// Returns `true` if `self` has a negative sign, including `-0.0`, NaNs with
    /// negative sign bit and negative infinity. Note that IEEE-745 doesn't assign any
    /// meaning to the sign bit in case of a NaN, and as Rust doesn't guarantee that
    /// the bit pattern of NaNs are conserved over arithmetic operations, the result of
    /// `is_sign_negative` on a NaN might produce an unexpected result in some cases.
    /// See [explanation of NaN as a special value](f32) for more info.
    #[must_use]
    #[inline]
    pub const fn is_sign_negative(self) -> bool {
        // IEEE754 says: isSignMinus(x) is true if and only if x has negative sign. isSignMinus
        // applies to zeros and NaNs as well.
        self.sign() == 1
    }

    #[must_use]
    #[inline]
    #[doc(hidden)]
    pub fn is_negative(self) -> bool {
        self.is_sign_negative()
    }

    /// Takes the reciprocal (inverse) of a number, `1/x`.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    #[inline]
    pub fn recip(self) -> f256 {
        1.0 / self
    }

    /// Converts radians to degrees.
    // #[must_use = "this returns the result of the operation, \
    //               without modifying the original"]
    // #[inline]
    // pub fn to_degrees(self) -> f256 {
    // The division here is correctly rounded with respect to the true
    // value of 180/π. (This differs from f32, where a constant must be
    // used to ensure a correctly rounded result.)
    //     self * (180.0f256 / consts::PI)
    // }

    /// Converts degrees to radians.
    // #[must_use = "this returns the result of the operation, \
    //               without modifying the original"]
    // #[inline]
    // pub fn to_radians(self) -> f256 {
    //     let value: f256 = consts::PI;
    //     self * (value / 180.0)
    // }

    /// Returns the maximum of the two numbers, ignoring NaN.
    ///
    /// If one of the arguments is NaN, then the other argument is returned.
    /// This follows the IEEE-754 2008 semantics for maxNum, except for handling of signaling NaNs;
    /// this function handles all NaNs the same way and avoids maxNum's problems with associativity.
    /// This also matches the behavior of libm’s fmax.
    #[must_use = "this returns the result of the comparison, without modifying either input"]
    #[inline]
    pub fn max(self, other: f256) -> f256 {
        unimplemented!()
    }

    /// Returns the minimum of the two numbers, ignoring NaN.
    ///
    /// If one of the arguments is NaN, then the other argument is returned.
    /// This follows the IEEE-754 2008 semantics for minNum, except for handling of signaling NaNs;
    /// this function handles all NaNs the same way and avoids minNum's problems with associativity.
    /// This also matches the behavior of libm’s fmin.
    #[must_use = "this returns the result of the comparison, without modifying either input"]
    #[inline]
    pub fn min(self, other: f256) -> f256 {
        unimplemented!()
    }

    /// Returns the maximum of the two numbers, propagating NaN.
    ///
    /// This returns NaN when *either* argument is NaN, as opposed to
    /// [`f256::max`] which only returns NaN when *both* arguments are NaN.
    ///
    /// If one of the arguments is NaN, then NaN is returned. Otherwise this returns the greater
    /// of the two numbers. For this operation, -0.0 is considered to be less than +0.0.
    /// Note that this follows the semantics specified in IEEE 754-2019.
    ///
    /// Also note that "propagation" of NaNs here doesn't necessarily mean that the bitpattern of a NaN
    /// operand is conserved; see [explanation of NaN as a special value](f32) for more info.
    #[must_use = "this returns the result of the comparison, without modifying either input"]
    #[inline]
    pub fn maximum(self, other: f256) -> f256 {
        if self > other {
            self
        } else if other > self {
            other
        } else if self == other {
            if self.is_sign_positive() && other.is_sign_negative() {
                self
            } else {
                other
            }
        } else {
            self + other
        }
    }

    /// Returns the minimum of the two numbers, propagating NaN.
    ///
    /// This returns NaN when *either* argument is NaN, as opposed to
    /// [`f256::min`] which only returns NaN when *both* arguments are NaN.
    ///
    /// If one of the arguments is NaN, then NaN is returned. Otherwise this returns the lesser
    /// of the two numbers. For this operation, -0.0 is considered to be less than +0.0.
    /// Note that this follows the semantics specified in IEEE 754-2019.
    ///
    /// Also note that "propagation" of NaNs here doesn't necessarily mean that the bitpattern of a NaN
    /// operand is conserved; see [explanation of NaN as a special value](f32) for more info.
    #[must_use = "this returns the result of the comparison, without modifying either input"]
    #[inline]
    pub fn minimum(self, other: f256) -> f256 {
        if self < other {
            self
        } else if other < self {
            other
        } else if self == other {
            if self.is_sign_negative() && other.is_sign_positive() {
                self
            } else {
                other
            }
        } else {
            self + other
        }
    }

    /// Raw transmutation to `u64`.
    ///
    /// This is currently identical to `transmute::<f256, u64>(self)` on all platforms.
    ///
    /// See [`from_bits`](Self::from_bits) for some discussion of the
    /// portability of this operation (there are almost no issues).
    ///
    /// Note that this function is distinct from `as` casting, which attempts to
    /// preserve the *numeric* value, and not the bitwise value.
    // #[must_use = "this returns the result of the operation, \
    //               without modifying the original"]
    // #[stable(feature = "float_bits_conv", since = "1.20.0")]
    // #[rustc_const_unstable(feature = "const_float_bits_conv", issue = "72447")]
    // #[inline]
    // pub const fn to_bits(self) -> u64 {
    //     // SAFETY: `u64` is a plain old datatype so we can always transmute to it.
    //     // ...sorta.
    //     //
    //     // See the SAFETY comment in f256::from_bits for more.
    //     #[rustc_const_unstable(feature = "const_float_bits_conv", issue = "72447")]
    //     const fn ct_f256_to_u64(ct: f256) -> u64 {
    //         match ct.classify() {
    //             FpCategory::Nan => {
    //                 panic!("const-eval error: cannot use f256::to_bits on a NaN")
    //             }
    //             FpCategory::Subnormal => {
    //                 panic!("const-eval error: cannot use f256::to_bits on a subnormal number")
    //             }
    //             FpCategory::Infinite | FpCategory::Normal | FpCategory::Zero => {
    //                 // SAFETY: We have a normal floating point number. Now we transmute, i.e. do a bitcopy.
    //                 unsafe { mem::transmute::<f256, u64>(ct) }
    //             }
    //         }
    //     }
    //     // SAFETY: `u64` is a plain old datatype so we can always... uh...
    //     // ...look, just pretend you forgot what you just read.
    //     // Stability concerns.
    //     let rt_f256_to_u64 = |rt| unsafe { mem::transmute::<f256, u64>(rt) };
    //     // SAFETY: We use internal implementations that either always work or fail at compile time.
    //     unsafe { intrinsics::const_eval_select((self,), ct_f256_to_u64, rt_f256_to_u64) }
    // }

    /// Raw transmutation from `u64`.
    ///
    /// This is currently identical to `transmute::<u64, f256>(v)` on all platforms.
    /// It turns out this is incredibly portable, for two reasons:
    ///
    /// * Floats and Ints have the same endianness on all supported platforms.
    /// * IEEE-754 very precisely specifies the bit layout of floats.
    ///
    /// However there is one caveat: prior to the 2008 version of IEEE-754, how
    /// to interpret the NaN signaling bit wasn't actually specified. Most platforms
    /// (notably x86 and ARM) picked the interpretation that was ultimately
    /// standardized in 2008, but some didn't (notably MIPS). As a result, all
    /// signaling NaNs on MIPS are quiet NaNs on x86, and vice-versa.
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
    // to be subject to floating point modes that alter nonzero subnormal numbers
    // to zero on reads and writes, aka "denormals are zero" and "flush to zero".
    // This is not a problem usually, but at least one tier2 platform for Rust
    // actually exhibits an FTZ behavior by default: thumbv7neon
    // aka "the Neon FPU in AArch32 state"
    //
    // Even with this, not all instructions exhibit the FTZ behaviors on thumbv7neon,
    // so this should load the same bits if LLVM emits the "correct" instructions,
    // but LLVM sometimes makes interesting choices about float optimization,
    // and other FPUs may do similar. Thus, it is wise to indulge luxuriously in caution.
    //
    // In addition, on x86 targets with SSE or SSE2 disabled and the x87 FPU enabled,
    // i.e. not soft-float, the way Rust does parameter passing can actually alter
    // a number that is "not infinity" to have the same exponent as infinity,
    // in a slightly unpredictable manner.
    //
    // And, of course evaluating to a NaN value is fairly nondeterministic.
    // More precisely: when NaN should be returned is knowable, but which NaN?
    // So far that's defined by a combination of LLVM and the CPU, not Rust.
    // This function, however, allows observing the bitstring of a NaN,
    // thus introspection on CTFE.
    //
    // In order to preserve, at least for the moment, const-to-runtime equivalence,
    // reject any of these possible situations from happening.
    // #[rustc_const_unstable(feature = "const_float_bits_conv", issue = "72447")]
    //     const fn ct_u64_to_f256(ct: u64) -> f256 {
    //         match f256::classify_bits(ct) {
    //             FpCategory::Subnormal => {
    //                 panic!("const-eval error: cannot use f256::from_bits on a subnormal number")
    //             }
    //             FpCategory::Nan => {
    //                 panic!("const-eval error: cannot use f256::from_bits on NaN")
    //             }
    //             FpCategory::Infinite | FpCategory::Normal | FpCategory::Zero => {
    //                 // SAFETY: It's not a frumious number
    //                 unsafe { mem::transmute::<u64, f256>(ct) }
    //             }
    //         }
    //     }
    //     // SAFETY: `u64` is a plain old datatype so we can always... uh...
    //     // ...look, just pretend you forgot what you just read.
    //     // Stability concerns.
    //     let rt_u64_to_f256 = |rt| unsafe { mem::transmute::<u64, f256>(rt) };
    //     // SAFETY: We use internal implementations that either always work or fail at compile time.
    //     unsafe { intrinsics::const_eval_select((v,), ct_u64_to_f256, rt_u64_to_f256) }
    // }

    /// Return the memory representation of this floating point number as a byte array in
    /// big-endian (network) byte order.
    ///
    /// See [`from_bits`](Self::from_bits) for some discussion of the
    /// portability of this operation (there are almost no issues).
    #[must_use = "this returns the result of the operation, \
                  without modifying the original"]
    #[inline]
    pub const fn to_be_bytes(self) -> [u8; 8] {
        self.to_bits().to_be_bytes()
    }

    /// Return the memory representation of this floating point number as a byte array in
    /// little-endian byte order.
    ///
    /// See [`from_bits`](Self::from_bits) for some discussion of the
    /// portability of this operation (there are almost no issues).
    #[must_use = "this returns the result of the operation, \
                  without modifying the original"]
    #[inline]
    pub const fn to_le_bytes(self) -> [u8; 8] {
        self.to_bits().to_le_bytes()
    }

    /// Return the memory representation of this floating point number as a byte array in
    /// native byte order.
    ///
    /// As the target platform's native endianness is used, portable code
    /// should use [`to_be_bytes`] or [`to_le_bytes`], as appropriate, instead.
    ///
    /// [`to_be_bytes`]: f256::to_be_bytes
    /// [`to_le_bytes`]: f256::to_le_bytes
    ///
    /// See [`from_bits`](Self::from_bits) for some discussion of the
    /// portability of this operation (there are almost no issues).
    #[must_use = "this returns the result of the operation, \
                  without modifying the original"]
    #[inline]
    pub const fn to_ne_bytes(self) -> [u8; 8] {
        self.to_bits().to_ne_bytes()
    }

    /// Create a floating point value from its representation as a byte array in big endian.
    ///
    /// See [`from_bits`](Self::from_bits) for some discussion of the
    /// portability of this operation (there are almost no issues).
    ///
    /// # Examples
    ///
    /// ```
    /// let value = f256::from_be_bytes([0x40, 0x29, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
    /// assert_eq!(value, 12.5);
    /// ```
    #[must_use]
    #[inline]
    pub const fn from_be_bytes(bytes: [u8; 8]) -> Self {
        unimplemented!()
    }

    /// Create a floating point value from its representation as a byte array in little endian.
    ///
    /// See [`from_bits`](Self::from_bits) for some discussion of the
    /// portability of this operation (there are almost no issues).
    ///
    /// # Examples
    ///
    /// ```
    /// let value = f256::from_le_bytes([0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x29, 0x40]);
    /// assert_eq!(value, 12.5);
    /// ```
    #[stable(feature = "float_to_from_bytes", since = "1.40.0")]
    #[rustc_const_unstable(feature = "const_float_bits_conv", issue = "72447")]
    #[must_use]
    #[inline]
    pub const fn from_le_bytes(bytes: [u8; 8]) -> Self {
        unimplemented!()
    }

    /// Create a floating point value from its representation as a byte array in native endian.
    ///
    /// As the target platform's native endianness is used, portable code
    /// likely wants to use [`from_be_bytes`] or [`from_le_bytes`], as
    /// appropriate instead.
    ///
    /// [`from_be_bytes`]: f256::from_be_bytes
    /// [`from_le_bytes`]: f256::from_le_bytes
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
    #[stable(feature = "float_to_from_bytes", since = "1.40.0")]
    #[rustc_const_unstable(feature = "const_float_bits_conv", issue = "72447")]
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
    /// floating point standard. The values are ordered in the following sequence:
    ///
    /// - negative quiet NaN
    /// - negative signaling NaN
    /// - negative infinity
    /// - negative numbers
    /// - negative subnormal numbers
    /// - negative zero
    /// - positive zero
    /// - positive subnormal numbers
    /// - positive numbers
    /// - positive infinity
    /// - positive signaling NaN
    /// - positive quiet NaN.
    ///
    /// The ordering established by this function does not always agree with the
    /// [`PartialOrd`] and [`PartialEq`] implementations of `f256`. For example,
    /// they consider negative and positive zero equal, while `total_cmp`
    /// doesn't.
    ///
    /// The interpretation of the signaling NaN bit follows the definition in
    /// the IEEE 754 standard, which may not match the interpretation by some of
    /// the older, non-conformant (e.g. MIPS) hardware implementations.
    #[must_use]
    #[inline]
    pub fn total_cmp(&self, other: &Self) -> crate::cmp::Ordering {
        let mut left = self.to_bits() as i64;
        let mut right = other.to_bits() as i64;

        // In case of negatives, flip all the bits except the sign
        // to achieve a similar layout as two's complement integers
        //
        // Why does this work? IEEE 754 floats consist of three fields:
        // Sign bit, exponent and mantissa. The set of exponent and mantissa
        // fields as a whole have the property that their bitwise order is
        // equal to the numeric magnitude where the magnitude is defined.
        // The magnitude is not normally defined on NaN values, but
        // IEEE 754 totalOrder defines the NaN values also to follow the
        // bitwise order. This leads to order explained in the doc comment.
        // However, the representation of magnitude is the same for negative
        // and positive numbers – only the sign bit is different.
        // To easily compare the floats as signed integers, we need to
        // flip the exponent and mantissa bits in case of negative numbers.
        // We effectively convert the numbers to "two's complement" form.
        //
        // To do the flipping, we construct a mask and XOR against it.
        // We branchlessly calculate an "all-ones except for the sign bit"
        // mask from negative-signed values: right shifting sign-extends
        // the integer, so we "fill" the mask with sign bits, and then
        // convert to unsigned to push one more zero bit.
        // On positive values, the mask is all zeros, so it's a no-op.
        left ^= (((left >> 63) as u64) >> 1) as i64;
        right ^= (((right >> 63) as u64) >> 1) as i64;

        left.cmp(&right)
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
    #[must_use = "method returns a new number and does not mutate the original value"]
    #[inline]
    pub fn clamp(self, min: f256, max: f256) -> f256 {
        assert!(min <= max);
        let mut x = self;
        if x < min {
            x = min;
        }
        if x > max {
            x = max;
        }
        x
    }

    fn decode(&self) -> RawFloat {
        RawFloat {
            fraction: u256 {
                hi: (self.bits.hi & HI_FRACTION_MASK),
                lo: self.bits.lo,
            },
            exponent: ((self.bits.hi & HI_EXP_MASK) >> HI_FRACTION_BITS) as i32
                - EXP_BIAS as i32
                - FRACTION_BITS as i32,
            normalized: false,
        }
    }
}
