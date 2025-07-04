// ---------------------------------------------------------------------------
// Copyright:   (c) 2022 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use core::{
    cmp::{max, min},
    ops::{Mul, MulAssign},
};

use crate::{
    abs_bits, abs_bits_sticky, exp_bits, f256, left_adj_signif, norm_bit,
    signif, BigUInt, BinEncAnySpecial, HiLo, EMAX, EMIN, EXP_BIAS, EXP_BITS,
    EXP_MAX, FRACTION_BITS, HI_ABS_MASK, HI_FRACTION_BIAS, HI_FRACTION_BITS,
    HI_FRACTION_MASK, HI_SIGN_MASK, INF_HI, MAX_HI, SIGNIFICAND_BITS,
    TOTAL_BITS, U256, U512,
};

#[inline]
#[allow(clippy::cast_possible_truncation)]
fn mul_signifs(x: &U256, y: &U256) -> (U256, u32, u32) {
    debug_assert!(x.hi.0 >= HI_SIGN_MASK);
    debug_assert!(y.hi.0 >= HI_SIGN_MASK);
    let (lo, mut hi) = x.widening_mul(y);
    let carry = (hi.hi.0 >= HI_SIGN_MASK) as u32;
    let shift = EXP_BITS - 1 + carry;
    let rem = hi.rem_pow2(shift).lo;
    let rnd_bits = (rem.0 >> (shift - 2)) as u32
        | (rem.0 > (1 << (shift - 1))) as u32
        | !lo.is_zero() as u32;
    hi >>= shift;
    (hi, carry, rnd_bits)
}

#[allow(clippy::cast_sign_loss)]
#[allow(clippy::cast_possible_wrap)]
#[allow(clippy::cast_possible_truncation)]
pub(crate) fn mul_abs_finite(
    abs_bits_x: &U256,
    abs_bits_y: &U256,
) -> (U256, u32) {
    // Extract biased exponents and normalized significands.
    let mut exp_bits_x = exp_bits(abs_bits_x) as i32;
    let norm_bit_x = norm_bit(abs_bits_x) as i32;
    let (mut norm_signif_x, norm_shift_x) = left_adj_signif(abs_bits_x);
    let mut exp_bits_y = exp_bits(abs_bits_y) as i32;
    let norm_bit_y = norm_bit(abs_bits_y) as i32;
    let (mut norm_signif_y, norm_shift_y) = left_adj_signif(abs_bits_y);

    // Calculate |x| * |y|.
    let (mut signif_z, carry, mut rnd_bits) =
        mul_signifs(&norm_signif_x, &norm_signif_y);
    const EMIN_EXTRA_SHIFT_BIAS: i32 = EMIN + 2 * EXP_BITS as i32;
    let mut exp_bits_z_minus_1 = (exp_bits_x - norm_bit_x)
        + (exp_bits_y - norm_bit_y)
        - (norm_shift_x + norm_shift_y) as i32
        + EMIN_EXTRA_SHIFT_BIAS
        + carry as i32;

    // If the result overflows the range of values representable as `f256`,
    // return +/- Infinity.
    if exp_bits_z_minus_1 >= (EXP_MAX - 1) as i32 {
        return (U256::new(INF_HI, 0), 0_u32);
    }

    // If the calculated biased exponent <= 0, the result may be subnormal or
    // underflow to ZERO.
    if exp_bits_z_minus_1 < 0 {
        let shift = exp_bits_z_minus_1.unsigned_abs();
        if shift > SIGNIFICAND_BITS + 1 {
            // Result underflows to zero.
            return (U256::ZERO, 0_u32);
        }
        if shift > 0 {
            // Adjust the rounding bits for correct final rounding.
            match shift {
                1 => {
                    rnd_bits = (((signif_z.lo.0 & 1) as u32) << 1)
                        | (rnd_bits != 0) as u32;
                }
                2 => {
                    rnd_bits =
                        ((signif_z.lo.0 & 3) as u32) | (rnd_bits != 0) as u32;
                }
                3..=127 => {
                    let rem = signif_z.rem_pow2(shift).lo.0;
                    rnd_bits = (rem >> (shift - 2)) as u32
                        | (rem > (1_u128 << (shift - 1))) as u32
                        | (rnd_bits != 0) as u32;
                }
                _ => {
                    let rem = signif_z.rem_pow2(shift);
                    rnd_bits = (rem >> (shift - 2)).lo.0 as u32
                        | (rem > (U256::ONE << (shift - 1))) as u32
                        | (rnd_bits != 0) as u32;
                }
            }
            signif_z >>= shift;
        }
        exp_bits_z_minus_1 = 0;
    }

    // Assemble the result.
    let abs_bits_z = U256::new(
        signif_z.hi.0 + ((exp_bits_z_minus_1 as u128) << HI_FRACTION_BITS),
        signif_z.lo.0,
    );
    (abs_bits_z, rnd_bits)
}

/// Compute z = x * y, rounded tie to even.
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_possible_wrap)]
#[allow(clippy::cast_sign_loss)]
#[inline]
pub(crate) fn mul(x: f256, y: f256) -> f256 {
    // The products sign is the XOR of the signs of the operands.
    let sign_bits_hi_z = (x.bits.hi.0 ^ y.bits.hi.0) & HI_SIGN_MASK;
    let mut abs_bits_x = abs_bits(&x);
    let mut abs_bits_y = abs_bits(&y);
    // Check whether one or both operands are NaN, infinite or zero.
    // We mask off the sign bit and mark subnormals having a significand less
    // than 2¹²⁸ in least bit of the representations high u128. This allows to
    // use only that part for the handling of special cases.
    let abs_bits_sticky_x = abs_bits_sticky(&abs_bits_x);
    let abs_bits_sticky_y = abs_bits_sticky(&abs_bits_y);
    if (abs_bits_sticky_x, abs_bits_sticky_y).any_special() {
        let max_abs_bits_sticky = max(abs_bits_sticky_x, abs_bits_sticky_y);
        let min_abs_bits_sticky = min(abs_bits_sticky_x, abs_bits_sticky_y);
        if min_abs_bits_sticky == 0 {
            // Atleast one operand is zero.
            if max_abs_bits_sticky < INF_HI {
                // ±0 × ±finite or ±finite × ±0
                return f256 {
                    bits: U256::new(sign_bits_hi_z, 0),
                };
            };
            if max_abs_bits_sticky == INF_HI {
                // ±0 × ±Inf or ±Inf × ±0
                return f256::NAN;
            }
        }
        if max_abs_bits_sticky > INF_HI {
            // Atleast one operand is NAN.
            return f256::NAN;
        }
        // Atleast one operand is infinite and the other non-zero.
        return f256 {
            bits: U256::new(sign_bits_hi_z | INF_HI, 0),
        };
    }

    // Both operands are finite and non-zero.
    let (mut bits_z, rnd_bits) = mul_abs_finite(&abs_bits_x, &abs_bits_y);
    bits_z.hi.0 |= sign_bits_hi_z;

    // Final rounding. Possibly overflowing into the exponent, but that is ok.
    if rnd_bits > 0b10 || (rnd_bits == 0b10 && bits_z.lo.is_odd()) {
        bits_z.incr();
    }
    f256 { bits: bits_z }
}

impl Mul for f256 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        mul(self, rhs)
    }
}

forward_ref_binop!(impl Mul, mul);

forward_op_assign!(impl MulAssign, mul_assign, Mul, mul);
