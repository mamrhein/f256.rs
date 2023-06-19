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
    ops::{Div, DivAssign},
};

use crate::{
    abs_bits, abs_bits_sticky,
    big_uint::{u512, DivRem},
    exp_bits, f256, norm_bit, norm_signif, u256, BinEncAnySpecial, EMIN,
    EXP_BIAS, EXP_BITS, EXP_MAX, FRACTION_BITS, HI_ABS_MASK, HI_EXP_MASK,
    HI_FRACTION_BIAS, HI_FRACTION_BITS, HI_FRACTION_MASK, HI_SIGN_MASK,
    INF_HI, MAX_HI, SIGNIFICAND_BITS,
};

#[inline]
fn div_signifs(x: &u256, y: &u256) -> (u256, u32) {
    debug_assert_eq!(x.hi.leading_zeros(), EXP_BITS);
    debug_assert_eq!(y.hi.leading_zeros(), EXP_BITS);
    let mut t = u512::new(u256::ZERO, *x);
    t <<= SIGNIFICAND_BITS + (x < y) as u32;
    let (mut q, r) = t.div_rem(y);
    let c = ((q.lo.lo & 1) as u32) << 1 | (!r.is_zero() as u32);
    q >>= 1;
    debug_assert!(q.hi.is_zero());
    (q.lo, c)
}

// Compute z = x / y, rounded tie to even.
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_possible_wrap)]
#[allow(clippy::cast_sign_loss)]
#[inline]
pub(crate) fn div(x: f256, y: f256) -> f256 {
    // The quotients sign is the XOR of the signs of the operands.
    let sign_bits_hi_z = (x.bits.hi ^ y.bits.hi) & HI_SIGN_MASK;
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
        if max_abs_bits_sticky > HI_EXP_MASK
            || abs_bits_sticky_x == abs_bits_sticky_y
        {
            // Atleast one operand is NAN or we have ±0 / ±0 or ±Inf / ±Inf.
            return f256::NAN;
        }
        if abs_bits_sticky_x < abs_bits_sticky_y {
            // ±0 / ±Inf or ±0 / ±finite or ±finite / ±Inf.
            return f256 {
                bits: u256::new(sign_bits_hi_z, 0),
            };
        }
        // ±Inf / ±0 or ±finite / ±0 or ±Inf / ±finite.
        return f256 {
            bits: u256::new(sign_bits_hi_z | INF_HI, 0),
        };
    }

    // Both operands are finite and non-zero.

    // Extract biased exponents and normalized significands.
    let mut exp_bits_x = exp_bits(&abs_bits_x) as i32;
    let mut exp_bits_y = exp_bits(&abs_bits_y) as i32;
    let norm_bit_x = norm_bit(&abs_bits_x) as i32;
    let norm_bit_y = norm_bit(&abs_bits_y) as i32;
    let (mut norm_signif_x, norm_shift_x) = norm_signif(&abs_bits_x);
    let (mut norm_signif_y, norm_shift_y) = norm_signif(&abs_bits_y);

    // Calculate |x| / |y|.
    let mut exp_bits_z_minus_1 = (exp_bits_x - norm_bit_x)
        - (exp_bits_y - norm_bit_y)
        - (norm_shift_x as i32 - norm_shift_y as i32)
        + (-EMIN - 1)
        + (norm_signif_x >= norm_signif_y) as i32;
    // If the result overflows the range of values representable as `f256`,
    // return +/- Infinity.
    if exp_bits_z_minus_1 >= EXP_MAX as i32 - 1 {
        return f256 {
            bits: u256::new(sign_bits_hi_z | INF_HI, 0),
        };
    }
    let (mut signif_z, mut rnd_bits) =
        div_signifs(&norm_signif_x, &norm_signif_y);

    // If the calculated biased exponent <= 0, the result may be subnormal or
    // underflow to ZERO.
    if exp_bits_z_minus_1 < 0 {
        let shift = exp_bits_z_minus_1.unsigned_abs();
        if shift > SIGNIFICAND_BITS + 1 {
            // Result underflows to zero.
            return f256 {
                bits: u256::new(sign_bits_hi_z, 0),
            };
        }
        if shift > 0 {
            // Adjust the rounding bits for correct final rounding.
            match shift {
                1 => {
                    rnd_bits = (((signif_z.lo & 1) as u32) << 1)
                        | (rnd_bits != 0) as u32;
                }
                2 => {
                    rnd_bits =
                        ((signif_z.lo & 3) as u32) | (rnd_bits != 0) as u32;
                }
                3..=127 => {
                    let rem = signif_z.rem_pow2(shift).lo;
                    rnd_bits = (rem >> (shift - 2)) as u32
                        | (rem > (1_u128 << (shift - 1))) as u32
                        | (rnd_bits != 0) as u32;
                }
                _ => {
                    let rem = signif_z.rem_pow2(shift);
                    rnd_bits = (&rem >> (shift - 2)).lo as u32
                        | (rem > (&u256::ONE << (shift - 1))) as u32
                        | (rnd_bits != 0) as u32;
                }
            }
            signif_z >>= shift;
        }
        exp_bits_z_minus_1 = 0;
    }

    // Assemble the result.
    let mut bits_z = u256::new(
        signif_z.hi + ((exp_bits_z_minus_1 as u128) << HI_FRACTION_BITS),
        signif_z.lo,
    );
    bits_z.hi |= sign_bits_hi_z;

    // Final rounding. Possibly overflowing into the exponent, but that is ok.
    if rnd_bits > 0b10 || (rnd_bits == 0b10 && ((bits_z.lo & 1) == 1)) {
        bits_z.incr();
    }
    f256 { bits: bits_z }
}

impl Div for f256 {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        div(self, rhs)
    }
}

forward_ref_binop!(impl Div, div);

forward_op_assign!(impl DivAssign, div_assign, Div, div);
