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

use super::{common::AsciiNumLit, Decimal, MAX_DIGITS};
use crate::{
    f256, EMAX, EMIN, EXP_BIAS, FRACTION_BITS, HI_FRACTION_BIAS,
    SIGNIFICAND_BITS,
};

// Parse a valid non-zero decimal number.
#[allow(unsafe_code)]
fn parse_decimal(s: &str) -> Decimal {
    let mut lit = AsciiNumLit::new(s.as_ref());
    let mut res = Decimal::default();

    res.sign = lit.get_sign();

    lit.skip_leading_zeroes(true);

    // Parse significant digits.
    lit.state.start_pos_signif = lit.len();
    while let Some(c) = lit.first() {
        let d = c.wrapping_sub(b'0');
        if d < 10 {
            res.add_digit(d);
        } else if *c == b'.' {
            lit.state.pos_radix_point = Some(lit.len());
        } else {
            break;
        }
        // SAFETY: safe because of call to lit.first above
        unsafe {
            lit.skip_1();
        }
    }
    lit.state.end_pos_signif = lit.len();

    // Check state.
    debug_assert!(!lit.state.invalid);
    let (mut n_digits, mut n_frac_digits) =
        if let Some(pos) = lit.state.pos_radix_point {
            (
                lit.state.start_pos_signif
                    - (pos < lit.state.start_pos_signif) as usize
                    - lit.state.end_pos_signif,
                pos - 1 - lit.state.end_pos_signif,
            )
        } else {
            (lit.state.start_pos_signif - lit.state.end_pos_signif, 0)
        };
    debug_assert_ne!(n_digits, 0);
    debug_assert_eq!(n_digits, res.n_digits);
    res.decimal_point = n_digits as i32 - n_frac_digits as i32;

    // check for explicit exponent
    let mut exponent = match lit.parse_exponent() {
        Some(exp) => exp,
        None => {
            unreachable!();
        }
    };
    res.decimal_point += exponent;

    // Normalize result.
    res.n_digits = min(res.n_digits, MAX_DIGITS);
    res.trim_trailing_zeroes();
    debug_assert!(
        res.n_digits == 0
            || res.digits[0] != 0 && res.digits[res.n_digits - 1] != 0
    );
    res
}

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
    let mut dec = parse_decimal(s);

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_frac_only_with_trailing_zeroes() {
        let s = "-.000000000000000010000000000000000000000000000000000000\
            00000000000000000000000000000000000000000000000000000000000000000";
        let mut dec = parse_decimal(s);
        assert_eq!(dec.n_digits, 1);
        assert_eq!(dec.digits[0], 1);
        assert_eq!(dec.decimal_point, -16);
        assert!(!dec.truncated);
    }

    #[test]
    fn parse_frac_only_without_trailing_zeroes() {
        let s = ".3000001";
        let mut dec = parse_decimal(s);
        assert_eq!(dec.n_digits, 7);
        assert_eq!(dec.digits[0], 3);
        assert_eq!(dec.digits[6], 1);
        assert_eq!(dec.decimal_point, 0);
        assert!(!dec.truncated);
    }

    #[test]
    fn parse_nonzero_digits_with_dot_and_trailing_zeroes_without_exp() {
        let s = "-750.629394531250000000000000000000000000000000000000";
        let mut dec = parse_decimal(s);
        assert_eq!(dec.n_digits, 14);
        assert_eq!(dec.digits[0], 7);
        assert_eq!(dec.digits[13], 5);
        assert_eq!(dec.decimal_point, 3);
        assert!(!dec.truncated);
    }

    #[test]
    fn parse_nonzero_digits_with_dot_and_trailing_zeroes_and_exp() {
        let s = "-7.62939453125000000000000000000000000000000000000000000\
            0000000000000000000000000000000000000000000000000000000000000000000\
            00e-06";
        let mut dec = parse_decimal(s);
        assert_eq!(dec.n_digits, 12);
        assert_eq!(dec.digits[0], 7);
        assert_eq!(dec.digits[11], 5);
        assert_eq!(dec.decimal_point, -5);
        assert!(!dec.truncated);
    }

    #[test]
    fn parse_max_digits_with_trailing_zeroes() {
        let mut s = "1.".to_string();
        s.push_str(&*"0".repeat(MAX_DIGITS - 2));
        s.push('9');
        s.push('0');
        s.push('0');
        let s = s.as_str();
        let mut dec = parse_decimal(s);
        assert_eq!(dec.n_digits, MAX_DIGITS);
        assert_eq!(dec.digits[0], 1);
        assert_eq!(dec.digits[MAX_DIGITS - 1], 9);
        assert_eq!(dec.decimal_point, 1);
        assert!(!dec.truncated);
    }

    #[test]
    fn parse_max_digits_exceeded() {
        let mut s = "1.".to_string();
        s.push_str(&*"0".repeat(MAX_DIGITS - 1));
        s.push('9');
        let s = s.as_str();
        let mut dec = parse_decimal(s);
        assert_eq!(dec.n_digits, 1);
        assert_eq!(dec.digits[0], 1);
        assert_eq!(dec.decimal_point, 1);
        assert!(dec.truncated);
    }
}
