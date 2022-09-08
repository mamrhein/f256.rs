// ---------------------------------------------------------------------------
// Copyright:   (c) 2022 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use core::{
    cmp::{min, Ordering},
    num::FpCategory,
    ops::Neg,
};

use crate::uint256::u256;

/// Precision level in relation to single precision float (f32) = 8
pub(crate) const PREC_LEVEL: u32 = 8;
/// Total number of bits = 256
pub(crate) const TOTAL_BITS: u32 = 1_u32 << PREC_LEVEL;
/// Number of exponent bits = 19
pub(crate) const EXP_BITS: u32 = 4 * PREC_LEVEL - 13;
/// Number of significand bits = 237
pub(crate) const SIGNIFICAND_BITS: u32 = TOTAL_BITS - EXP_BITS;
/// Number of fraction bits = 236
pub(crate) const FRACTION_BITS: u32 = SIGNIFICAND_BITS - 1;
/// Maximum value of biased base 2 exponent = 0x7ffff = 524287
pub(crate) const EXP_MAX: u32 = (1_u32 << EXP_BITS) - 1;
/// Base 2 exponent bias = 0x3ffff = 262143
pub(crate) const EXP_BIAS: u32 = EXP_MAX >> 1;
/// Maximum value of base 2 exponent = 0x3ffff = 262143
pub(crate) const EMAX: i32 = (EXP_MAX >> 1) as i32;
/// Minimum value of base 2 exponent = -262142
pub(crate) const EMIN: i32 = 1 - EMAX;
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

#[derive(Clone, Copy, Debug, Eq, Ord)]
pub(crate) struct Float256Repr {
    pub(crate) bits: u256,
}

impl Float256Repr {
    /// The difference between `1.0` and the next larger representable number:
    /// 2⁻²³⁶ ≈ 9.055679e-72.
    pub(crate) const EPSILON: Self = Self {
        bits: u256 {
            hi: EPSILON_HI,
            lo: 0,
        },
    };

    /// Largest finite value:  2²⁶²¹⁴⁴ − 2²⁶¹⁹⁰⁷ ≈ 1.6113e78913.
    pub(crate) const MAX: Self = Self {
        bits: u256 {
            hi: MAX_HI,
            lo: u128::MAX,
        },
    };

    /// Smallest finite `f256` value: 2²⁶¹⁹⁰⁷ - 2²⁶²¹⁴⁴ ≈ -1.6113e78913.
    pub(crate) const MIN: Self = Self::MAX.neg();

    /// Smallest positive normal `f256` value: 2⁻²⁶²¹⁴² ≈ 2.4824e−78913.
    pub(crate) const MIN_POSITIVE: Self = {
        Self {
            bits: u256 {
                hi: HI_FRACTION_BIAS,
                lo: 0,
            },
        }
    };

    /// Not a Number (NaN).
    pub(crate) const NAN: Self = {
        Self {
            bits: u256 { hi: NAN_HI, lo: 0 },
        }
    };

    /// Infinity (∞).
    pub(crate) const INFINITY: Self = {
        Self {
            bits: u256 { hi: INF_HI, lo: 0 },
        }
    };

    /// Negative infinity (−∞).
    pub(crate) const NEG_INFINITY: Self = {
        Self {
            bits: u256 {
                hi: NEG_INF_HI,
                lo: 0,
            },
        }
    };

    /// Additive identity
    pub(crate) const ZERO: Self = Self {
        bits: u256 { hi: 0, lo: 0 },
    };

    /// Negative additive identity
    pub(crate) const NEG_ZERO: Self = Self::ZERO.neg();

    /// Multiplicative identity
    pub(crate) const ONE: Self = Self {
        bits: u256 {
            hi: (EXP_BIAS as u128) << HI_FRACTION_BITS,
            lo: 0,
        },
    };

    /// Multiplicative negator
    pub(crate) const NEG_ONE: Self = Self::ONE.neg();

    /// Equivalent of 2.0: 2 × ONE.
    pub(crate) const TWO: Self = Self {
        bits: u256 {
            hi: ((1 + EXP_BIAS) as u128) << HI_FRACTION_BITS,
            lo: 0,
        },
    };

    /// Raw assembly from significand, exponent and sign.
    #[inline]
    pub(crate) const fn new(
        significand: u256,
        exponent: u32,
        sign: u32,
    ) -> Self {
        Self {
            bits: u256 {
                hi: (significand.hi & HI_FRACTION_MASK)
                    | ((exponent as u128) << HI_FRACTION_BITS)
                    | ((sign as u128) << HI_SIGN_SHIFT),
                lo: significand.lo,
            },
        }
    }

    /// Raw transmutation from `[u64; 4]` (in native endian order).
    #[inline]
    pub(crate) const fn from_bits(bits: [u64; 4]) -> Self {
        Self {
            bits: u256::from_bits(bits),
        }
    }

    /// Create a floating point value from its representation as a byte array in
    /// big endian.
    #[must_use]
    #[inline]
    #[allow(unsafe_code)]
    pub(crate) const fn from_be_bytes(bytes: [u8; 32]) -> Self {
        // safe because size of [[u8; 16]; 2] == [u8; 32]
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
    pub(crate) const fn from_le_bytes(bytes: [u8; 32]) -> Self {
        // safe because size of [[u8; 16]; 2] == [u8; 32]
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
    pub(crate) const fn from_ne_bytes(bytes: [u8; 32]) -> Self {
        // safe because size of [u64; 4] == [u8; 32]
        let bits: [u64; 4] = unsafe { core::mem::transmute(bytes) };
        Self::from_bits(bits)
    }

    /// Construct a finite, non-zero `Float256Repr` f from sign s, exponent t
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
    pub(crate) fn encode(
        sign: u32,
        mut exponent: i32,
        mut significand: u256,
    ) -> Self {
        debug_assert!(sign == 0 || sign == 1);
        debug_assert!(
            exponent >= EMIN - FRACTION_BITS as i32
                && exponent <= EMAX - FRACTION_BITS as i32
        );
        debug_assert!(!significand.is_zero());
        // We have an integer based representation `(-1)ˢ × 2ᵗ × c` and need to
        // transform it into a fraction based representation
        // `(-1)ˢ × 2ᵉ × (1 + m × 2¹⁻ᵖ)`, where `Eₘᵢₙ <= e <= Eₘₐₓ` and
        // `0 < m < 2ᵖ⁻¹` or `(-1)ˢ × 2ᵉ × m × 2¹⁻ᵖ`, where `e = Eₘᵢₙ - 1` and
        // `0 < m < 2ᵖ⁻¹`.

        // 1. Compensate radix shift
        exponent += FRACTION_BITS as i32;
        // 2. Normalize significand
        let nlz = significand.leading_zeros();
        match nlz.cmp(&EXP_BITS) {
            Ordering::Greater => {
                // shift left
                let shift = (nlz - EXP_BITS) as usize;
                if exponent >= EMIN + shift as i32 {
                    significand <<= shift;
                    exponent -= shift as i32;
                } else {
                    // Number is subnormal
                    significand <<= (exponent - EMIN) as usize;
                    exponent = EMIN - 1;
                }
            }
            Ordering::Less => {
                // shift right and round
                let shift = (EXP_BITS - nlz) as usize;
                exponent += shift as i32;
                significand.idiv_pow2(shift as u32);
                // Rounding may have caused significand to overflow.
                if (significand.hi >> HI_FRACTION_BITS + 1) != 0 {
                    exponent += 1;
                    significand >>= 1;
                }
            }
            _ => {}
        }
        // 3. Offset exponent
        let biased_exponent = exponent + EXP_BIAS as i32;
        debug_assert!(biased_exponent >= 0);
        Self {
            bits: u256 {
                hi: (sign as u128) << HI_SIGN_SHIFT
                    | ((biased_exponent as u128) << HI_FRACTION_BITS)
                    | (significand.hi & HI_FRACTION_MASK),
                lo: significand.lo,
            },
        }
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
    pub(crate) const fn exponent(&self) -> i32 {
        self.biased_exponent() as i32 - EXP_BIAS as i32
    }

    /// Returns the fraction of `self`.
    /// Pre-condition: `self` is finite!
    #[inline]
    pub(crate) const fn fraction(&self) -> u256 {
        debug_assert!(
            self.is_finite(),
            "Attempt to extract fraction from Infinity or NaN."
        );
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

    /// Extract sign, exponent and significand from `self`.
    // TODO: ensure that for all finite values: encode(x.decode()) == x
    #[inline]
    pub(crate) const fn decode(&self) -> (u32, i32, u256) {
        (self.sign(), self.exponent(), self.significand())
    }

    /// Raw transmutation to `[u64; 4]` (in native endian order).
    #[inline]
    pub(crate) const fn to_bits(&self) -> [u64; 4] {
        self.bits.to_bits()
    }

    /// Return the memory representation of this floating point number as a byte
    /// array in big-endian (network) byte order.
    #[must_use]
    #[inline]
    #[allow(unsafe_code)]
    pub(crate) const fn to_be_bytes(self) -> [u8; 32] {
        let bytes = [self.bits.hi.to_be_bytes(), self.bits.lo.to_be_bytes()];
        // safe because size of [[u8; 16]; 2] == size of [u8; 32]
        unsafe { core::mem::transmute(bytes) }
    }

    /// Return the memory representation of this floating point number as a byte
    /// array in little-endian byte order.
    #[must_use]
    #[inline]
    #[allow(unsafe_code)]
    pub(crate) const fn to_le_bytes(self) -> [u8; 32] {
        let bytes = [self.bits.lo.to_le_bytes(), self.bits.hi.to_le_bytes()];
        // safe because size of [[u8; 16]; 2] == size of [u8; 32]
        unsafe { core::mem::transmute(bytes) }
    }

    /// Return the memory representation of this floating point number as a byte
    /// array in native byte order.
    #[must_use]
    #[inline]
    #[allow(unsafe_code)]
    pub(crate) const fn to_ne_bytes(self) -> [u8; 32] {
        let bytes = self.to_bits();
        // safe because size of [u64; 4] == size of [u8; 32]
        unsafe { core::mem::transmute(bytes) }
    }

    /// Returns `true` if this value is NaN.
    #[must_use]
    #[inline]
    pub(crate) const fn is_nan(self) -> bool {
        (self.bits.hi & HI_ABS_MASK) > HI_EXP_MASK
            || ((self.bits.hi & HI_ABS_MASK) == HI_EXP_MASK
                && self.bits.lo != 0)
    }

    /// Returns `true` if this value is positive infinity or negative infinity,
    /// and `false` otherwise.
    #[must_use]
    #[inline]
    pub(crate) const fn is_infinite(self) -> bool {
        (self.bits.hi & HI_ABS_MASK) == HI_EXP_MASK && self.bits.lo == 0
    }

    /// Returns `true` if this number is neither infinite nor NaN.
    #[must_use]
    #[inline]
    pub(crate) const fn is_finite(self) -> bool {
        (self.bits.hi & HI_EXP_MASK) != HI_EXP_MASK
    }

    /// Returns `true` if the number is subnormal.
    #[must_use]
    #[inline]
    pub(crate) const fn is_subnormal(self) -> bool {
        (self.bits.hi & HI_EXP_MASK) == 0 && !self.is_zero()
    }

    /// Returns `true` if the number is neither zero, infinite,
    /// subnormal, or NaN.
    #[must_use]
    #[inline]
    pub(crate) const fn is_normal(self) -> bool {
        // normal => exponent ∉ {0, EXP_MAX} => ((exponent + 1) & EXP_MAX) > 1
        ((self.biased_exponent() + 1) & EXP_MAX) > 1
    }

    /// Returns `true` if `self` is equal to `+0.0` or `-0.0`.
    #[must_use]
    #[inline]
    pub(crate) const fn is_zero(self) -> bool {
        (self.bits.hi << 1) == 0 && self.bits.lo == 0
    }

    /// Returns `true` if `self` is either not a number, infinite or equal to
    /// zero.
    #[must_use]
    #[inline]
    pub(crate) const fn is_special(self) -> bool {
        self.is_zero() || (self.bits.hi & HI_EXP_MASK) == INF_HI
    }

    /// Returns the floating point category of the represented number.
    pub(crate) const fn classify(&self) -> FpCategory {
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

    /// Returns `true` if `self` has a positive sign, including `+0.0`, positive
    /// infinity and NaN.
    #[must_use]
    #[inline]
    pub(crate) const fn is_sign_positive(self) -> bool {
        !self.is_sign_negative()
    }

    /// Returns `true` if `self` has a negative sign, including `-0.0`and
    /// negative infinity.
    #[must_use]
    #[inline]
    pub(crate) const fn is_sign_negative(self) -> bool {
        self.sign() == 1
    }

    #[inline]
    pub(crate) const fn abs(self) -> Self {
        Self {
            bits: u256 {
                hi: self.bits.hi & HI_SIGN_MASK,
                lo: self.bits.lo,
            },
        }
    }

    #[inline]
    pub(crate) const fn neg(self) -> Self {
        Self {
            bits: u256 {
                hi: self.bits.hi ^ HI_SIGN_MASK,
                lo: self.bits.lo,
            },
        }
    }

    #[inline]
    pub(crate) fn set_sign(&mut self, sign: u32) {
        self.bits.hi |= (sign as u128) << HI_SIGN_SHIFT;
    }
}

impl Neg for Float256Repr {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self::Output {
        self.neg()
    }
}

// Note: Float256Repr instances are treated as equal if and only if their
// raw binary values are equal. I. e. - other than with f256 - NAN.repr ==
// NAN.repr and ZERO.repr != NEG_ZERO.repr!
impl PartialEq for Float256Repr {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.bits == other.bits
    }
}

impl PartialOrd for Float256Repr {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // The internal representation of floats does - besides their sign -
        // gives a total ordering following the intended mathematical ordering.
        // Thus, flipping the sign bit allows to compare the raw values.
        // Note that this differs from f256. See doc of fn f256::total_cmp.
        self.neg().bits.partial_cmp(&(*other).neg().bits)
    }
}

#[inline]
pub(crate) fn add_special(x: Float256Repr, y: Float256Repr) -> Float256Repr {
    // Either x or y or both are either not a number, infinite or equal to zero.
    if x.is_zero() {
        return if y.is_zero() {
            if x.is_sign_negative() && y.is_sign_negative() {
                x
            } else {
                Float256Repr::ZERO
            }
        } else {
            y
        };
    }
    if y.is_zero() {
        return x;
    }
    if x.is_nan() || y.is_nan() {
        return Float256Repr::NAN;
    }
    if x.is_infinite() {
        return if (x.bits.hi ^ y.bits.hi) == HI_SIGN_MASK {
            // x and y are infinite and have different signs
            Float256Repr::NAN
        } else {
            x
        };
    }
    // x is a number and y is infinite
    y
}

type InplaceBinOp = fn(&mut u256, &u256);
const SIGNIF_OPS: [InplaceBinOp; 2] = [u256::iadd, u256::isub];

#[inline]
pub(crate) fn add(x: Float256Repr, y: Float256Repr) -> Float256Repr {
    // Both operands are finite and non-zero.
    // Compare the absolute values of the operands and swap them in case x < y.
    let mut a: Float256Repr;
    let mut b: Float256Repr;
    if x.abs() >= y.abs() {
        a = x;
        b = y;
    } else {
        a = y;
        b = x;
    }
    // Extract biased exponents and significands (shifted left by 3 bits to give
    // room for a round, guard and sticky bit). These shifts are safe because
    // the significands use at most 237 bits in an u256.
    let mut a_exp = a.biased_exponent();
    let b_exp = b.biased_exponent();
    let mut a_signif = a.significand() << 3;
    let mut b_signif = b.significand() << 3;
    // Here a >= b => a_exp >= b_exp => a_exp - b_exp >= 0.
    // We adjust the significand of b by right-shifting it.
    // We limit the adjustment by an upper limit of SIGNIFICAND_BITS + 2. Thus,
    // the silent bit of b's significant is atmost to the position of the sticky
    // bit. Any further shift would have no effect on the result.
    let adj = min(a_exp - b_exp, SIGNIFICAND_BITS + 2);
    let sticky_bit =
        !(adj == 0 || (b_signif << (u256::BITS - adj) as usize).is_zero());
    b_signif >>= adj as usize;
    b_signif.lo |= sticky_bit as u128;
    // Determine the actual op to be performed: if the sign of the operands
    // differ, it's a subtraction, otherwise an addition.
    let signs_differ = (((x.bits.hi ^ y.bits.hi) & HI_SIGN_MASK) != 0);
    let op = SIGNIF_OPS[signs_differ as usize];
    op(&mut a_signif, &b_signif);
    // If addition carried over, right-shift the significand and increment
    // the exponent.
    if (a_signif.hi >> (HI_FRACTION_BITS + 4)) != 0 {
        a_signif >>= 1;
        a_signif.lo |= sticky_bit as u128;
        a_exp += 1;
    } else {
        // If subtraction cancelled the hidden bit or x and y are subnormal,
        // left-shift the significand and decrement the exponent. We limit the
        // adjustment to avoid the biased exponent to become negative.
        let adj = min(SIGNIFICAND_BITS + 3 - a_signif.leading_zeros(), a_exp);
        a_signif <<= adj as usize;
        a_exp -= adj;
    }
    // If the result overflows the range of values representable as `f256`,
    // return +/- Infinity.
    if a_exp >= EXP_MAX {
        return [Float256Repr::INFINITY, Float256Repr::NEG_INFINITY]
            [a.sign() as usize];
    }
    // Determine carry from round, guard and sticky bit.
    let carry = (a_signif.lo & 0x7_u128) > 0x4;
    // Shift significand back, erase hidden bit and set exponent and sign.
    let mut bits = a_signif >> 3;
    bits.hi &= HI_FRACTION_MASK;
    bits.hi |= (a_exp as u128) << HI_FRACTION_BITS;
    bits.hi |= a.bits.hi & HI_SIGN_MASK;
    // Final rounding. Possibly overflowing into the exponent, but that is ok.
    if carry {
        bits.incr();
    }
    Float256Repr { bits }
}
