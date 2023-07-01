// ---------------------------------------------------------------------------
// Copyright:   (c) 2023 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use core::{
    cmp::{max, min},
    fmt::{Debug, Formatter},
    ops::BitXor,
};
use std::ops::ShlAssign;

use crate::{
    abs_bits, abs_bits_sticky,
    big_uint::{u256, u512, BigIntHelper, BigUIntHelper},
    binops::mul::mul_abs_finite,
    exp_bits, f256, left_adj_signif, norm_bit, norm_signif, sign_bits_hi,
    signif, BinEncAnySpecial, EMIN, EXP_BIAS, EXP_BITS, FRACTION_BITS,
    HI_FRACTION_BITS, HI_SIGN_MASK, INF_HI, MAX_HI, SIGNIFICAND_BITS,
};

/// Helper type representing signed integers of 768 bits.
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Default, Eq, Ord, PartialOrd, PartialEq)]
struct u768 {
    hi: u256,
    mi: u256,
    lo: u256,
}

impl u768 {
    const BITS: u32 = 768;
    const STICKY_BIT: Self = Self {
        hi: u256::ZERO,
        mi: u256::ZERO,
        lo: u256::ONE,
    };

    fn new(hi: &u256, mi: &u256, lo: &u256) -> Self {
        Self {
            hi: *hi,
            mi: *mi,
            lo: *lo,
        }
    }

    fn from_u256_shifted(u: &u256, mut shr: u32) -> Self {
        debug_assert!(shr < Self::BITS);
        match shr {
            0 => Self::new(&u, &u256::ZERO, &u256::ZERO),
            1..=255 => {
                Self::new(&(u >> shr), &(u << (256 - shr)), &u256::ZERO)
            }
            256 => Self::new(&u256::ZERO, u, &u256::ZERO),
            257..=511 => Self::new(
                &u256::ZERO,
                &(u >> (shr - 256)),
                &(u << (512 - shr)),
            ),
            512 => Self::new(&u256::ZERO, &u256::ZERO, u),
            513..=Self::BITS => {
                shr -= 512;
                let sticky = !(u << (256 - shr)).is_zero() as u128;
                let mut lo = (u >> shr);
                lo.lo |= sticky;
                Self::new(&u256::ZERO, &u256::ZERO, &lo)
            }
            _ => unreachable!(),
        }
    }

    fn invert(&mut self) {
        self.lo.lo = self.lo.lo.bitxor(u128::MAX);
        self.lo.hi = self.lo.hi.bitxor(u128::MAX);
        self.mi.lo = self.mi.lo.bitxor(u128::MAX);
        self.mi.hi = self.mi.hi.bitxor(u128::MAX);
        self.hi.lo = self.hi.lo.bitxor(u128::MAX);
        self.hi.hi = self.hi.hi.bitxor(u128::MAX);
    }

    fn iadd(&mut self, rhs: &Self) {
        let mut carry = false;
        (self.lo, carry) = self.lo.overflowing_add(&rhs.lo);
        (self.mi, carry) = self.mi.carrying_add(&rhs.mi, carry);
        (self.hi, carry) = self.hi.carrying_add(&rhs.hi, carry);
    }

    fn isub(&mut self, rhs: &Self) {
        let mut borrow = false;
        (self.lo, borrow) = self.lo.overflowing_sub(&rhs.lo);
        (self.mi, borrow) = self.mi.borrowing_sub(&rhs.mi, borrow);
        (self.hi, borrow) = self.hi.borrowing_sub(&rhs.hi, borrow);
    }

    fn leading_zeros(&self) -> u32 {
        let hi_is_zero = self.hi.is_zero();
        let hi_and_mi_are_zero = hi_is_zero && self.mi.is_zero();
        self.hi.leading_zeros()
            + hi_is_zero as u32 * self.mi.leading_zeros()
            + hi_and_mi_are_zero as u32 * self.lo.leading_zeros()
    }
}

impl Debug for u768 {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "[{:032x} {:032x} {:032x} {:032x} {:032x} {:032x}]",
            self.hi.hi,
            self.hi.lo,
            self.mi.hi,
            self.mi.lo,
            self.lo.hi,
            self.lo.lo,
        )
    }
}

impl ShlAssign<u32> for u768 {
    fn shl_assign(&mut self, mut rhs: u32) {
        assert!(rhs < Self::BITS, "Attempt to shift left with overflow.");
        match rhs {
            1..=255 => {
                let mut carry = u256::ZERO;
                (self.lo, carry) = self.lo.widening_shl(rhs);
                (self.mi, carry) = self.mi.carrying_shl(rhs, &carry);
                (self.hi, _) = self.hi.carrying_shl(rhs, &carry);
            }
            256 => {
                self.hi = self.mi;
                self.mi = self.lo;
                self.lo = u256::ZERO;
            }
            257..=511 => {
                rhs -= 256;
                let (t, mut carry) = self.lo.widening_shl(rhs);
                (self.hi, carry) = self.mi.carrying_shl(rhs, &carry);
                self.mi = t;
                self.lo = u256::ZERO;
            }
            512 => {
                self.hi = self.lo;
                self.mi = u256::ZERO;
                self.lo = u256::ZERO;
            }
            513..=767 => {
                self.hi = &self.lo << (rhs - 512);
                self.mi = u256::ZERO;
                self.lo = u256::ZERO;
            }
            0 => {}
            _ => unreachable!(),
        }
    }
}

/// Compute z = x * y + a, only once rounded tie to even.
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_possible_wrap)]
#[allow(clippy::cast_sign_loss)]
#[inline]
pub(crate) fn fma(x: f256, y: f256, a: f256) -> f256 {
    // The products sign is the XOR of the signs of the operands.
    let sign_bits_hi_p = (x.bits.hi ^ y.bits.hi) & HI_SIGN_MASK;
    let sign_bits_hi_a = sign_bits_hi(&a);

    // Check whether one or more operands are NaN, infinite or zero.
    // We mask off the sign bit and mark subnormals having a significand less
    // than 2¹²⁸ in least bit of the representations high u128. This allows to
    // use only that part for the handling of special cases.
    let mut abs_bits_x = abs_bits(&x);
    let abs_bits_sticky_x = abs_bits_sticky(&abs_bits_x);
    let mut abs_bits_y = abs_bits(&y);
    let abs_bits_sticky_y = abs_bits_sticky(&abs_bits_y);
    let mut abs_bits_a = abs_bits(&a);
    let abs_bits_sticky_a = abs_bits_sticky(&abs_bits_a);
    if (abs_bits_sticky_x, abs_bits_sticky_y, abs_bits_sticky_a).any_special()
    {
        // At least one operand is zero, infinite or nan.
        let max_abs_bits_sticky_xy =
            max(abs_bits_sticky_x, abs_bits_sticky_y);
        let min_abs_bits_sticky_xy =
            min(abs_bits_sticky_x, abs_bits_sticky_y);
        if max_abs_bits_sticky_xy > INF_HI || abs_bits_sticky_a > INF_HI {
            // Atleast one operand is nan => result is nan.
            return f256::NAN;
        }
        // All operands are numbers.
        if abs_bits_sticky_a == 0 {
            // The addend is zero.
            if min_abs_bits_sticky_xy == 0 {
                // Atleast one multiplicand is zero.
                if max_abs_bits_sticky_xy < INF_HI {
                    // The other is finite => product is zero
                    // => result is zero.
                    return f256 {
                        bits: u256::new(sign_bits_hi_p & sign_bits_hi_a, 0),
                    };
                };
                if max_abs_bits_sticky_xy == INF_HI {
                    // The other is infinite => product is nan
                    // => result is nan.
                    return f256::NAN;
                }
            }
            if max_abs_bits_sticky_xy < INF_HI {
                // Both multiplicands are finite => result = product.
                let (mut bits_z, rnd_bits) =
                    mul_abs_finite(&abs_bits_x, &abs_bits_y);
                bits_z.hi |= sign_bits_hi_p;
                // Final rounding. Possibly overflowing into the exponent, but
                // that is ok.
                if rnd_bits > 0b10
                    || (rnd_bits == 0b10 && ((bits_z.lo & 1) == 1))
                {
                    bits_z.incr();
                }
                return f256 { bits: bits_z };
            }
        }
        if abs_bits_sticky_a == INF_HI {
            // The addend is infinite.
            if max_abs_bits_sticky_xy == INF_HI {
                // Atleast one multiplicand is infinite.
                if min_abs_bits_sticky_xy == 0 {
                    // The other is zero => product is nan
                    // => result is nan.
                    return f256::NAN;
                }
                // Product is infinite
                if sign_bits_hi_p == sign_bits_hi_a {
                    return a;
                } else {
                    return f256::NAN;
                }
            } else {
                // The product is finite => result is infinite.
                return a;
            };
        }
        if min_abs_bits_sticky_xy == 0 {
            // Atleast one multiplicand is zero.
            if max_abs_bits_sticky_xy < INF_HI {
                // The other is finite => product is zero => result = addend.
                return a;
            };
            if max_abs_bits_sticky_xy == INF_HI {
                // The other is infinite => product is nan => result is nan.
                return f256::NAN;
            }
        }
        if max_abs_bits_sticky_xy == INF_HI {
            // Atleast one multiplicand is infinite and the other non-zero
            // => product is infinite => result is infinite
            return f256 {
                bits: u256::new(sign_bits_hi_p | INF_HI, 0),
            };
        }
    }

    // All operands are finite and non-zero.
    assert!(!(abs_bits_sticky_x, abs_bits_sticky_y, abs_bits_sticky_a)
        .any_special());

    // Calculate x * y + a, rounded tie to even.

    // Extract biased exponents and significands.
    let exp_bits_x = exp_bits(&abs_bits_x) as i32;
    let norm_bit_x = norm_bit(&abs_bits_x) as i32;
    let signif_x = signif(&abs_bits_x);
    let exp_bits_y = exp_bits(&abs_bits_y) as i32;
    let norm_bit_y = norm_bit(&abs_bits_y) as i32;
    let signif_y = signif(&abs_bits_y);
    let exp_bits_a = exp_bits(&abs_bits_a) as i32;
    let norm_bit_a = norm_bit(&abs_bits_a) as i32;
    let signif_a = signif(&abs_bits_a);

    // The significand of product x * y has 2 * FRACTION_BITS = 472 fractional
    // bits. Shifting one operand 2 bits left, gets its radix point aligned at
    // bit 474 and its guarantied to have 2 trailing zeroes.
    let (lo, hi) = signif_x.widening_mul(&(&signif_y << 2));
    let mut signif_p = u768 {
        hi: u256::ZERO,
        mi: hi,
        lo,
    };
    // The products exponent before normalization is
    // e(p) = e(x)+ e(y)
    // and biased
    // E(p) = e(x) + e(y) - Eₘᵢₙ
    //      = E(x) - n(x) + Eₘᵢₙ + E(y) - n(y) + Eₘᵢₙ - Eₘᵢₙ
    //      = E(x) - n(x)) + E(y) - n(y) + Eₘᵢₙ
    let exp_bits_p =
        (exp_bits_x - norm_bit_x) + (exp_bits_y - norm_bit_y) + EMIN;
    // Align the addend based on the difference of the exponents of the addend
    // and the product. Swap the operands, so that the larger one is the
    // first. The sign of the result is the sign of the larger operand.
    let mut signif_a_shifted = u768::default();
    // The exponent difference
    // d = e(a) - e(p)
    //   = e(a) - e(x) - e(y)
    //   = E(a) - n(a) + Eₘᵢₙ - (E(x) - n(x) + Eₘᵢₙ) - (E(y) - n(y) + Eₘᵢₙ)
    //   = E(a) - n(a) + Eₘᵢₙ - (E(x) - n(x) + (E(y) - n(y) + Eₘᵢₙ) - Eₘᵢₙ
    //   = E(a) - n(a) - E(p)
    let d = exp_bits_a - norm_bit_a - exp_bits_p;
    const ADDEND_TOO_SMALL_UPPER_LIMIT: i32 =
        -2 * SIGNIFICAND_BITS as i32 + 1;
    const PROD_ANCHORED_LOWER_LIMIT: i32 = ADDEND_TOO_SMALL_UPPER_LIMIT + 1;
    const ADDEND_ANCHORED_UPPER_LIMIT: i32 = SIGNIFICAND_BITS as i32 + 2;
    // Starting point of the alignment is the addends significand as highest
    // u256 part of a u768 value. Its radix point is initially aligned at bit
    // 512 + FRACTION_BITS = 748. The initial offset between the radix points
    // of p and a is (512 + FRACTION_BITS) - (2 * FRACTION_BITS + 2).
    const REL_OFFSET: u32 = 512 - FRACTION_BITS - 2;
    let (mut signif_z, signif_t, sign_bits_hi_z) = match d {
        i32::MIN..=ADDEND_TOO_SMALL_UPPER_LIMIT => {
            (&mut signif_p, &u768::STICKY_BIT, sign_bits_hi_p)
        }
        PROD_ANCHORED_LOWER_LIMIT..=-1 => {
            let shr = (REL_OFFSET as i32 - d) as u32;
            signif_a_shifted = u768::from_u256_shifted(&signif_a, shr);
            (&mut signif_p, &signif_a_shifted, sign_bits_hi_p)
        }
        0..=1 => {
            // Because of the uncertainty on the leading digit of the product
            // we have to check explicitly for max(p, a).
            let shr = (REL_OFFSET as i32 - d) as u32;
            signif_a_shifted = u768::from_u256_shifted(&signif_a, shr);
            if signif_p >= signif_a_shifted {
                (&mut signif_p, &signif_a_shifted, sign_bits_hi_p)
            } else {
                (&mut signif_a_shifted, &signif_p, sign_bits_hi_a)
            }
        }
        2..=ADDEND_ANCHORED_UPPER_LIMIT => {
            let shr = (REL_OFFSET as i32 - d) as u32;
            signif_a_shifted = u768::from_u256_shifted(&signif_a, shr);
            (&mut signif_a_shifted, &signif_p, sign_bits_hi_a)
        }
        _ => {
            // Product too small.
            signif_a_shifted = u768::from_u256_shifted(&signif_a, 0);
            (&mut signif_a_shifted, &signif_p, sign_bits_hi_a)
        }
    };
    // Calculate |p + a|.
    if sign_bits_hi_p == sign_bits_hi_a {
        signif_z.iadd(signif_t);
    } else {
        signif_z.isub(signif_t);
    }
    let signif_z_nlz = signif_z.leading_zeros();
    if signif_z_nlz == u768::BITS {
        return f256::ZERO;
    }
    // Calculate exponent and normalize result.
    let (exp_bits_m1_z, shl) = if d <= ADDEND_ANCHORED_UPPER_LIMIT {
        let n = signif_z_nlz - EXP_BITS;
        let carry = REL_OFFSET as i32 - n as i32;
        let t = exp_bits_p + carry + 1;
        if t >= 1 {
            (t - 1, n)
        } else {
            (0, REL_OFFSET)
        }
    } else {
        (exp_bits_a - 1, 0)
    };
    *signif_z <<= shl;
    // Now we have the results preliminary significand in signif_z.hi, before
    // rounding.
    debug_assert!(signif_z.hi.leading_zeros() >= EXP_BITS);
    let (hi_bits, carry) = signif_z.mi.hi.widening_shr(u128::BITS - 3);
    let rnd_bits = hi_bits as u32
        | (carry != 0 || signif_z.mi.lo != 0 || !signif_z.lo.is_zero())
            as u32;
    let mut bits_z = u256::new(
        signif_z.hi.hi + ((exp_bits_m1_z as u128) << HI_FRACTION_BITS),
        signif_z.hi.lo,
    );
    // Final rounding. Possibly overflowing into the exponent, but that is ok.
    if rnd_bits > 0x4 || rnd_bits == 0x4 && (bits_z.lo & 1) == 1 {
        bits_z.incr();
    }
    bits_z.hi |= sign_bits_hi_z;
    f256 { bits: bits_z }
}
