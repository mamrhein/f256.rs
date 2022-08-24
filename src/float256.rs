// ---------------------------------------------------------------------------
// Copyright:   (c) 2022 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use core::num::FpCategory;

use crate::{rawfloat::RawFloat, uint256::u256};

/// Precision level in relation to single precision float (f32) = 8
pub(crate) const PREC_LEVEL: u32 = 8;
/// Total number of bits = 256
pub(crate) const TOTAL_BITS: u32 = 1_u32 << PREC_LEVEL;
/// Number of exponent bits = 19
pub(crate) const EXP_BITS: u32 = 4 * PREC_LEVEL - 13;
/// Maximum value of biased base 2 exponent = 0x7ffff = 524287
pub(crate) const EXP_MAX: u32 = (1_u32 << EXP_BITS) - 1;
/// Base 2 exponent bias = 0x3ffff = 262143
pub(crate) const EXP_BIAS: u32 = EXP_MAX >> 1;
/// Maximum value of base 2 exponent = 0x3ffff = 262143
pub(crate) const EMAX: i32 = EXP_BIAS as i32;
/// Minimum value of base 2 exponent = -262142
pub(crate) const EMIN: i32 = 1 - EMAX;
/// Number of fraction bits = 236
pub(crate) const FRACTION_BITS: u32 = TOTAL_BITS - EXP_BITS - 1;
/// Number of bits in hi u128
pub(crate) const HI_TOTAL_BITS: u32 = TOTAL_BITS >> 1;
/// Number of bits to shift right for sign = 127
pub(crate) const HI_SIGN_SHIFT: u32 = HI_TOTAL_BITS - 1;
/// Number of fraction bits in hi u128 = 108
pub(crate) const HI_FRACTION_BITS: u32 = FRACTION_BITS - HI_TOTAL_BITS;
/// Fraction bias in hi u128 = 1e108 = 0x1000000000000000000000000000
pub(crate) const HI_FRACTION_BIAS: u128 = 1_u128 << HI_FRACTION_BITS;
/// Fraction mask in hi u128 = 0xfffffffffffffffffffffffffff
pub(crate) const HI_FRACTION_MASK: u128 = HI_FRACTION_BIAS - 1;
/// Exponent mask in hi u128 = 0x7ffff000000000000000000000000000
pub(crate) const HI_EXP_MASK: u128 = (EXP_MAX as u128) << HI_FRACTION_BITS;
/// Sign mask in hi u128 = 0x80000000000000000000000000000000
pub(crate) const HI_SIGN_MASK: u128 = 1_u128 << HI_SIGN_SHIFT;
/// Value of hi u128 for NaN = 0x7ffff000000000000000000000000001
pub(crate) const NAN_HI: u128 = HI_EXP_MASK + 1;
/// Value of hi u128 for Inf = 0x7ffff000000000000000000000000000
pub(crate) const INF_HI: u128 = HI_EXP_MASK;
/// Value of hi u128 for -Inf = 0xfffff000000000000000000000000000
pub(crate) const NEG_INF_HI: u128 = HI_SIGN_MASK | HI_EXP_MASK;

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug)]
pub struct f256 {
    bits: u256,
}

impl f256 {
    /// The radix or base of the internal representation of `f256`.
    pub const RADIX: u32 = 2;

    /// Number of significant digits in base 2: 237.
    pub const MANTISSA_DIGITS: u32 = FRACTION_BITS + 1;

    /// Number of significant digits in base 2: 237.
    pub const SIGNIFICANT_DIGITS: u32 = FRACTION_BITS + 1;

    /// Approximate number of significant digits in base 10.
    pub const DIGITS: u32 = 71;

    /// The difference between `1.0` and the next larger representable number:
    /// 2^-236 ≈ 9.055679e-72.
    pub const EPSILON: Self = Self::from_bits(
        ((EXP_BIAS - FRACTION_BITS) as u128) << HI_FRACTION_BITS,
        0,
    );

    /// Smallest finite `f256` value: 2^261907 - 2^262144≈ -1.6113 × 10^78913.
    // TODO: replace by -MAX when Neg implemented
    pub const MIN: f256 = Self::from_bits(
        1 << HI_SIGN_SHIFT
            | ((EXP_MAX as u128 - 1) << HI_FRACTION_BITS)
            | HI_FRACTION_MASK,
        u128::MAX,
    );

    /// Smallest positive normal `f256` value: 2^−262142 ≈ 2.4824 × 10^−78913.
    pub const MIN_POSITIVE: f256 = Self::from_bits(HI_FRACTION_BIAS, 0);

    /// Largest finite `f256` value:  2^262144 − 2^261907 ≈ 1.6113 × 10^78913.
    pub const MAX: f256 = Self::from_bits(
        ((EXP_MAX as u128 - 1) << HI_FRACTION_BITS) | HI_FRACTION_MASK,
        u128::MAX,
    );

    /// One greater than the minimum possible normal power of 2 exponent:
    /// -262142.
    pub const MIN_EXP: i32 = EMIN;

    /// Maximum possible power of 2 exponent: 262143.
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
    /// This implementation does not make such a difference and uses exactly one
    /// bit pattern for NaN.
    pub const NAN: Self = Self::from_bits(NAN_HI, 0);

    /// Infinity (∞).
    pub const INFINITY: Self = Self::from_bits(INF_HI, 0);

    /// Negative infinity (−∞).
    pub const NEG_INFINITY: Self = Self::from_bits(NEG_INF_HI, 0);

    /// Additive identity
    pub const ZERO: Self = Self {
        bits: u256 { hi: 0, lo: 0 },
    };

    /// Multiplicative identity
    pub const ONE: Self = Self {
        bits: u256 {
            hi: (EXP_BIAS as u128) << HI_FRACTION_BITS,
            lo: 0,
        },
    };

    /// Multiplicative negator
    // TODO: replace by -ONE
    pub const NEG_ONE: Self = Self {
        bits: u256 {
            hi: 1 << HI_SIGN_SHIFT | (EXP_BIAS as u128) << HI_FRACTION_BITS,
            lo: 0,
        },
    };

    /// Equivalent of 2.0
    pub const TWO: Self = Self {
        bits: u256 {
            hi: ((1 + EXP_BIAS) as u128) << HI_FRACTION_BITS,
            lo: 0,
        },
    };

    /// TODO: Equivalent of 10.0
    // pub const TEN: Self = Self {
    //     bits: u256 {
    //         hi: ((1 + EXP_BIAS) as u128) << HI_FRACTION_BITS,
    //         lo: 0,
    //     },
    // };

    /// Returns the sign bit of `self`: 0 = positive, 1 = negative.
    #[inline]
    const fn sign(&self) -> u32 {
        (self.bits.hi >> HI_SIGN_SHIFT) as u32
    }

    /// Returns `true` if this value is NaN.
    #[must_use]
    #[inline]
    pub const fn is_nan(self) -> bool {
        self.bits.hi == NAN_HI
    }

    /// Returns `true` if this value is positive infinity or negative infinity,
    /// and `false` otherwise.
    #[must_use]
    #[inline]
    pub const fn is_infinite(self) -> bool {
        (self.bits.hi & HI_EXP_MASK) == HI_EXP_MASK
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
        matches!(self.classify(), FpCategory::Subnormal)
    }

    /// Returns `true` if the number is neither zero, infinite,
    /// subnormal, or NaN.
    #[must_use]
    #[inline]
    pub const fn is_normal(self) -> bool {
        matches!(self.classify(), FpCategory::Normal)
    }

    /// Returns the floating point category of the number. If only one property
    /// is going to be tested, it is generally faster to use the specific
    /// predicate instead.
    #[allow(unsafe_code)]
    pub const fn classify(&self) -> FpCategory {
        match (
            self.bits.hi & HI_EXP_MASK,
            self.bits.hi & HI_FRACTION_MASK,
            self.bits.lo,
        ) {
            (HI_EXP_MASK, 0, _) => FpCategory::Infinite,
            (HI_EXP_MASK, NAN_HI, _) => FpCategory::Nan,
            (0, 0, 0) => FpCategory::Zero,
            (0, ..) => FpCategory::Subnormal,
            _ => FpCategory::Normal,
        }
    }

    /// Returns `true` if `self` has a positive sign, including `+0.0`, positive
    /// infinity and NaN.
    #[must_use]
    #[inline]
    pub const fn is_sign_positive(self) -> bool {
        !self.is_sign_negative()
    }

    /// Returns `true` if `self` has a negative sign, including `-0.0`and
    /// negative infinity.
    #[must_use]
    #[inline]
    pub const fn is_sign_negative(self) -> bool {
        self.sign() == 1
    }

    /// Takes the reciprocal (inverse) of a number, `1/x`.
    // TODO: uncomment when Div implemented
    // #[must_use]
    // #[inline]
    // pub fn recip(self) -> f256 {
    //     Self::ONE / self
    // }

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
    /// This follows the IEEE-754 2008 semantics for maxNum, except for handling
    /// of signaling NaNs; this function handles all NaNs the same way and
    /// avoids maxNum's problems with associativity. This also matches the
    /// behavior of libm’s fmax.
    #[must_use]
    #[inline]
    pub fn max(self, other: f256) -> f256 {
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
    pub fn min(self, other: f256) -> f256 {
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
    pub fn maximum(self, other: f256) -> f256 {
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
    pub fn minimum(self, other: f256) -> f256 {
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
    // "72447")]     const fn ct_f256_to_u64(ct: f256) -> u64 {
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
    // "72447")]     const fn ct_u64_to_f256(ct: u64) -> f256 {
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
        Self {
            bits: u256 { hi, lo },
        }
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
    /// [`to_be_bytes`]: f256::to_be_bytes
    /// [`to_le_bytes`]: f256::to_le_bytes
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
    pub fn total_cmp(&self, other: &Self) -> core::cmp::Ordering {
        // let mut left = self.to_bits() as i64;
        // let mut right = other.to_bits() as i64;

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
        // left ^= (((left >> 63) as u64) >> 1) as i64;
        // right ^= (((right >> 63) as u64) >> 1) as i64;
        //
        // left.cmp(&right)
        unimplemented!()
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
    pub fn clamp(self, min: f256, max: f256) -> f256 {
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

    fn decode(&self) -> RawFloat {
        RawFloat {
            significand: u256 {
                hi: (self.bits.hi & HI_FRACTION_MASK),
                lo: self.bits.lo,
            },
            exponent: ((self.bits.hi & HI_EXP_MASK) >> HI_FRACTION_BITS) as i32
                - EXP_BIAS as i32
                - FRACTION_BITS as i32,
            normalized: false,
        }
    }

    fn encode(is_negative: bool, raw: &mut RawFloat) -> Self {
        if !raw.normalized {
            raw.normalize();
        }
        let biased_exp = raw.exponent + EXP_BIAS as i32 + FRACTION_BITS as i32;
        let shifted_exp = (biased_exp as u128) << HI_FRACTION_BITS;
        let hi = raw.significand.hi
            & shifted_exp
            & ((is_negative as u128) << HI_SIGN_SHIFT);
        let lo = raw.significand.lo;
        Self {
            bits: u256 { hi, lo },
        }
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
