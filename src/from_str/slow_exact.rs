// ---------------------------------------------------------------------------
// Copyright:   (c) 2022 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

/// Implementation of `Simple Decimal Conversion` as decribed in
/// `Nigel Tao: ParseNumberF64 by Simple Decimal Conversion`
/// [https://nigeltao.github.io/blog/2020/parse-number-f64-simple.html] for
/// `f256`.
use core::cmp::min;

use super::big_num::Decimal;
use crate::{
    f256, EMAX, EMIN, EXP_BIAS, FRACTION_BITS, HI_FRACTION_BIAS,
    SIGNIFICAND_BITS,
};

/// Create a correctly rounded `f256` from a valid decimal number literal.
pub(super) fn f256_exact(s: &str) -> f256 {
    // ⌊log₁₀(2⁶⁴-1)⌋
    const MAX_DEC_SHIFT: u8 = 19;
    // [0] + [⌊log₂(10ⁿ⌋] for n in [1..MAX_DEC_SHIFT - 1] + [60]
    const DEC_TO_BIN_SHIFT: [u32; (MAX_DEC_SHIFT + 1) as usize] = [
        0,
        3,
        6,
        9,
        13,
        16,
        19,
        23,
        26,
        29,
        33,
        36,
        39,
        43,
        46,
        49,
        53,
        56,
        59,
        Decimal::MAX_SHIFT,
    ];

    // Parse the number literal into a high precision decimal.
    let mut dec = Decimal::default();
    dec.parse(s);

    // Multiply / devide by powers of 2 (using non-rounding shifts) until the
    // number is in the range [½..1].
    let mut bin_exp = 0_i32;
    while dec.decimal_point > 0 {
        let n = DEC_TO_BIN_SHIFT
            [min(dec.decimal_point as usize, MAX_DEC_SHIFT as usize)];
        dec.right_shift(n);
        bin_exp += n as i32;
    }
    while dec.decimal_point <= 0 {
        let n = if dec.decimal_point == 0 {
            match dec.digits[0] {
                0..=1 => 2,
                2..=4 => 1,
                _ => break,
            }
        } else {
            DEC_TO_BIN_SHIFT
                [min(-dec.decimal_point as usize, MAX_DEC_SHIFT as usize)]
        };
        dec.left_shift(n);
        bin_exp -= n as i32;
    }
    // Adjust exponent to put the number in range [1..2].
    bin_exp -= 1;

    // If the exponent is too small, right-shift digits and adjust exponent.
    while bin_exp < EMIN {
        let n = min((EMIN - bin_exp) as usize, Decimal::MAX_SHIFT as usize);
        dec.right_shift(n as u32);
        bin_exp += n as i32;
    }

    // If the exponent is too large, return ±Infinity
    if bin_exp > EMAX {
        return [f256::INFINITY, f256::NEG_INFINITY][dec.sign as usize];
    }

    // Shift the number so that it's in range [2ᵖ..2ᵖ⁺¹) (or [2ᵖ⁻¹..2ᵖ) if
    // it's subnormal) and round it to the nearest integer to get the
    // significand.
    let mut sh = SIGNIFICAND_BITS;
    while sh > 0 {
        let n = min(sh, Decimal::MAX_SHIFT);
        sh -= n;
        dec.left_shift(n);
    }
    let mut significand = dec.round();
    if significand.hi >= HI_FRACTION_BIAS << 1 {
        // Rounding overflowed, need to shift back.
        dec.right_shift(1);
        bin_exp += 1;
        if bin_exp > EMAX {
            return [f256::INFINITY, f256::NEG_INFINITY][dec.sign as usize];
        }
        significand = dec.round();
    }
    // Adjust exponent if number is subnormal.
    if significand.hi < HI_FRACTION_BIAS {
        bin_exp -= 1;
    }
    if significand.is_zero() {
        return [f256::ZERO, f256::NEG_ZERO][dec.sign as usize];
    }

    debug_assert!(bin_exp >= -EMAX);
    debug_assert!(bin_exp <= EMAX);
    let biased_exponent = (EXP_BIAS as i32 + bin_exp) as u32;
    f256::new(significand, biased_exponent, dec.sign)
}
