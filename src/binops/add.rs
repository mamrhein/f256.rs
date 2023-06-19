// ---------------------------------------------------------------------------
// Copyright:   (c) 2022 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use core::{
    cmp::{max, min, Ordering},
    mem::swap,
    ops::{Add, AddAssign, Sub, SubAssign},
};

use crate::{
    abs_bits, abs_bits_sticky, exp_bits, f256, norm_bit, sign_bits_hi,
    signif, u256, BinEncAnySpecial, EXP_MAX, FRACTION_BITS, HI_ABS_MASK,
    HI_EXP_MASK, HI_FRACTION_BIAS, HI_FRACTION_BITS, HI_FRACTION_MASK,
    HI_SIGN_MASK, INF_HI, MAX_HI, SIGNIFICAND_BITS,
};

pub(crate) fn add(x: f256, y: f256) -> f256 {
    let mut abs_bits_x = abs_bits(&x);
    let mut abs_bits_y = abs_bits(&y);
    let mut sign_bits_hi_x = sign_bits_hi(&x);
    let mut sign_bits_hi_y = sign_bits_hi(&y);
    // Check whether one or both operands are NaN, infinite or zero.
    // We mask off the sign bit and mark subnormals having a significand less
    // than 2¹²⁸ in least bit of the representations high u128. This allows to
    // use only that part for the handling of special cases.
    let abs_bits_sticky_x = abs_bits_sticky(&abs_bits_x);
    let abs_bits_sticky_y = abs_bits_sticky(&abs_bits_y);
    if (abs_bits_sticky_x, abs_bits_sticky_y).any_special() {
        let max_abs_bits_sticky = max(abs_bits_sticky_x, abs_bits_sticky_y);
        if max_abs_bits_sticky == 0 {
            // Both operands are zero.
            return f256 {
                bits: u256::new(sign_bits_hi_x & sign_bits_hi_y, 0),
            };
        }
        if max_abs_bits_sticky > HI_EXP_MASK
            || (abs_bits_sticky_x == INF_HI
                && abs_bits_sticky_y == INF_HI
                && sign_bits_hi_x != sign_bits_hi_y)
        {
            // Atleast one operand is NAN, or both operands are infinite and
            // their signs differ.
            return f256::NAN;
        }
        // For all other special cases return the operand with the greater
        // absolute value.
        return if abs_bits_sticky_x > abs_bits_sticky_y {
            x
        } else {
            y
        };
    }

    // Both operands are finite and non-zero.

    // In case |x| = |y| and the sign(x) != sign(y), the result is +0.
    if abs_bits_x == abs_bits_y && sign_bits_hi_x != sign_bits_hi_y {
        return f256::ZERO;
    }

    // Compare the absolute values of the operands and swap them in case
    // |x| < |y|.
    if abs_bits_x < abs_bits_y {
        swap(&mut abs_bits_x, &mut abs_bits_y);
        swap(&mut sign_bits_hi_x, &mut sign_bits_hi_y);
    }

    // Extract biased exponents and significands.
    let mut exp_bits_x = exp_bits(&abs_bits_x);
    let exp_bits_y = exp_bits(&abs_bits_y);
    let norm_bit_x = norm_bit(&abs_bits_x);
    let norm_bit_y = norm_bit(&abs_bits_y);
    let mut signif_x = signif(&abs_bits_x);
    let mut signif_y = signif(&abs_bits_y);

    // Calculate |x + y|

    // Determine the actual op to be performed: if the sign of the operands
    // are equal, it's an addition, otherwise a subtraction.
    let op = if sign_bits_hi_x == sign_bits_hi_y {
        <&u256 as Add>::add
    } else {
        <&u256 as Sub>::sub
    };
    let mut abs_bits_z = if norm_bit_x == 0 {
        // x subnormal and |x| >= |y| => y subnormal.
        add_or_sub_subnormals(&signif_x, &signif_y, op)
    } else if exp_bits_x == exp_bits_y {
        // Exponents are equal, so there's no need for shifting and rounding.
        add_or_sub_normals_exact(exp_bits_x, &signif_x, &signif_y, op)
    } else {
        // Exponents and significands have to be adjusted and the result has
        // to be rounded.
        add_or_sub_rounded(
            exp_bits_x,
            exp_bits_y,
            &mut signif_x,
            &mut signif_y,
            op,
        )
    };

    // The sign of the result is the sign of the operand with the greater
    // absolute value.
    abs_bits_z.hi |= sign_bits_hi_x;
    f256 { bits: abs_bits_z }
}

#[inline]
fn add_or_sub_subnormals<'a>(
    signif_x: &'a u256,
    signif_y: &'a u256,
    op: fn(&'a u256, &'a u256) -> u256,
) -> u256 {
    // In case of two subnormals we don't have to care about overflow
    // because the overflow bit goes into the biased exponent, which is ok
    // because the result then is normal with an exponent equal to Eₘᵢₙ which
    // has the biased encoding = 1.
    op(signif_x, signif_y)
}

#[inline]
fn add_or_sub_normals_exact<'a>(
    mut exp_bits_z: u32,
    signif_x: &'a u256,
    signif_y: &'a u256,
    op: fn(&'a u256, &'a u256) -> u256,
) -> u256 {
    debug_assert!(exp_bits_z > 0);
    debug_assert!(signif_x >= signif_y);
    let mut abs_bits_z = op(signif_x, signif_y);
    // If addition carried over, adjust the significand and increment
    // the exponent.
    if abs_bits_z.hi >= HI_FRACTION_BIAS << 1 {
        exp_bits_z += 1;
        // If the result overflows the range of values representable as
        // `f256`, return +Inf.
        if exp_bits_z >= EXP_MAX {
            return u256::new(INF_HI, 0);
        }
        let l2bits = (abs_bits_z.lo & 3) as u32;
        abs_bits_z >>= 1;
        abs_bits_z += (l2bits == 3) as u128;
    }
    // If subtraction cancelled the hidden bit, left-shift the significand
    // and decrement the exponent, unless exp is already zero.
    else if abs_bits_z.hi < HI_FRACTION_BIAS {
        let adj = min(FRACTION_BITS - abs_bits_z.msb(), exp_bits_z);
        exp_bits_z -= adj;
        // exp == 0 => result is subnormal => no hidden bit
        abs_bits_z <<= (adj - (exp_bits_z == 0) as u32);
    }
    // Erase hidden bit and set exponent.
    abs_bits_z.hi &= HI_FRACTION_MASK;
    abs_bits_z.hi |= (exp_bits_z as u128) << HI_FRACTION_BITS;
    abs_bits_z
}

#[inline]
fn add_or_sub_rounded<'a>(
    exp_bits_x: u32,
    exp_bits_y: u32,
    signif_x: &'a mut u256,
    signif_y: &'a mut u256,
    op: fn(&'a u256, &'a u256) -> u256,
) -> u256 {
    debug_assert!(exp_bits_x > 0); // x is normal!
    debug_assert!(
        exp_bits_x > exp_bits_y
            || (exp_bits_x == exp_bits_y && signif_x > signif_y)
    ); // |x| > |y|

    // Shift significands by 3 bits to give room for a round, guard and sticky
    // bit. These shifts are safe because the significands use at most 237
    // bits in an u256.
    *signif_x <<= 3;
    *signif_y <<= 3;
    // |x| > |y| => exp_bits_x >= exp_bits_y => exp_bits_x - exp_bits_y >= 0.
    // We adjust the significand of y by right-shifting it.
    // We limit the adjustment by an upper limit of SIGNIFICAND_BITS + 2.
    // Thus, the silent bit of y's significand is atmost to the position
    // of the sticky bit. Any further shift would have no effect on the
    // result.
    let adj = min(
        exp_bits_x - exp_bits_y - (exp_bits_y == 0) as u32,
        SIGNIFICAND_BITS + 2,
    );
    let mut sticky_bit = if adj <= 3 {
        0_u128
    } else if adj >= SIGNIFICAND_BITS + 3 {
        1_u128
    } else {
        !(&*signif_y << (u256::BITS - adj)).is_zero() as u128
    };
    *signif_y >>= adj;
    signif_y.lo |= sticky_bit;

    // Add / subract the adjusted operands.
    let mut exp_bits_z = exp_bits_x;
    let mut abs_bits_z = op(signif_x, signif_y);
    // If addition carried over, right-shift the significand and increment
    // the exponent.
    if abs_bits_z.hi >= HI_FRACTION_BIAS << 4 {
        exp_bits_z += 1;
        // If the result overflows the range of values representable as
        // `f256`, return +Inf.
        if exp_bits_z >= EXP_MAX {
            return u256::new(INF_HI, 0);
        }
        sticky_bit |= abs_bits_z.lo & 1;
        abs_bits_z >>= 1;
        abs_bits_z.lo |= sticky_bit;
    }
    // If subtraction cancelled the hidden bit, left-shift the significand
    // and decrement the exponent.
    else if abs_bits_z.hi < HI_FRACTION_BIAS << 3 {
        let adj = min(FRACTION_BITS + 3 - abs_bits_z.msb(), exp_bits_z);
        exp_bits_z -= adj;
        // exp_bits_x == 0 => result is subnormal => no hidden bit
        abs_bits_z <<= (adj - (exp_bits_z == 0) as u32);
    }

    // Get round, guard and sticky bit.
    let l3bits = (abs_bits_z.lo & 0x7_u128) as u32;
    // Shift significand back, erase hidden bit and set exponent.
    abs_bits_z >>= 3;
    abs_bits_z.hi &= HI_FRACTION_MASK;
    abs_bits_z.hi |= (exp_bits_z as u128) << HI_FRACTION_BITS;
    // Final rounding. Possibly overflowing into the exponent, but that is ok.
    if l3bits > 0x4 || l3bits == 0x4 && (abs_bits_z.lo & 1) == 1 {
        abs_bits_z.incr();
    }
    abs_bits_z
}

impl Add for f256 {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        add(self, rhs)
    }
}

forward_ref_binop!(impl Add, add);

forward_op_assign!(impl AddAssign, add_assign, Add, add);

impl Sub for f256 {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        add(self, -rhs)
    }
}

forward_ref_binop!(impl Sub, sub);

forward_op_assign!(impl SubAssign, sub_assign, Sub, sub);
