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
    fmt,
    mem::MaybeUninit,
    ops::{AddAssign, Rem},
};

use super::{
    common::floor_log10_pow2,
    formatted::{Formatted, Part},
    pow10_div_pow2_lut::{
        get_segment_params, pow10_div_pow2, ADDITIONAL_BITS, CHUNK_CUTOFF,
        CHUNK_SIZE, COMPRESSION_RATE,
    },
};
use crate::{
    biguint::DivRem,
    f256,
    to_str::{
        common::floor_log10, dec_repr::DecNumRepr,
        powers_of_five::get_power_of_five,
    },
    u256, u512, EMAX, EMIN, FRACTION_BITS, SIGNIFICAND_BITS,
};

const CHUNK_BASE: u64 = 10_u64.pow(CHUNK_SIZE);
const SHIFT: u32 = ADDITIONAL_BITS + COMPRESSION_RATE;

#[derive(PartialEq)]
enum Round {
    Up,
    ToEven,
    Down,
}

/// Calculate ⌊x × y / 2ᵏ⌋ % B, where B = 10 ^ CHUNK_SIZE.
#[inline(always)]
fn mul_shift_mod(x: &u256, y: &u512, k: u32) -> u64 {
    debug_assert!(k > 256);
    let mut hi = x.widening_mul(&y.hi);
    let lo = x.widening_mul(&y.lo);
    hi.lo += &lo.hi;
    if hi.lo < lo.hi {
        hi.hi.incr();
    }
    hi >>= (k - 256);
    &hi % CHUNK_BASE
}

fn bin_fract_2_dec_str(
    signif2: u256,
    exp2: i32,
    prec: usize,
    buf: &mut String,
) -> Round {
    let mut round = Round::Down;
    let segment_idx =
        (-exp2 - SIGNIFICAND_BITS as i32) as u32 / COMPRESSION_RATE;
    let n_chunks = prec as u32 / CHUNK_SIZE + 1;
    let (n_zero_chunks, segment_shift) = get_segment_params(segment_idx);
    debug_assert!(segment_shift > exp2);
    let shift = (segment_shift - exp2) as u32;
    if n_chunks <= n_zero_chunks {
        buf.push_str("0".repeat(prec).as_str());
    } else {
        if n_zero_chunks > 0 {
            buf.push_str(
                "0".repeat((n_zero_chunks * CHUNK_SIZE) as usize).as_str(),
            );
        }
        let n_signif_chunks = n_chunks - n_zero_chunks;
        assert!(
            n_signif_chunks <= CHUNK_CUTOFF,
            "Internal limit for significant fractional digits exceeded."
        );
        for chunk_idx in 0..n_signif_chunks {
            let t = pow10_div_pow2(segment_idx, chunk_idx);
            let mut chunk = mul_shift_mod(&signif2, &t, shift);
            if chunk_idx < n_signif_chunks - 1 {
                buf.push_str(
                    format!("{:01$}", chunk, CHUNK_SIZE as usize).as_str(),
                );
            } else {
                // last chunk
                let n_digits = prec as u32 - (n_chunks - 1) * CHUNK_SIZE;
                debug_assert!(n_digits <= CHUNK_SIZE);
                let d = 10_u64.pow(CHUNK_SIZE - n_digits);
                let rem = chunk % d;
                chunk /= d;
                let tie = d >> 1;
                if rem > tie {
                    round = Round::Up;
                } else if rem == tie {
                    // Need to check whether we really have a tie, i.e.
                    // signif2 * 10 ^ (prec + 1) / 2 ^ -exp2 is an
                    // integer. This is the case if the number of
                    // trailing zeroes of the numerator is greater or
                    // equal to -exp2.
                    round = if (signif2.trailing_zeros() as i32)
                        >= (-exp2 - prec as i32 - 1)
                    {
                        Round::ToEven
                    } else {
                        Round::Up
                    };
                }
                if n_digits > 0 {
                    buf.push_str(
                        format!("{:01$}", chunk, n_digits as usize).as_str(),
                    );
                }
            }
        }
    }
    round
}

/// Round-Up the given fixed-point string representation of a decimal number.
#[allow(unsafe_code)]
#[inline]
fn round_up_fixed_point_inplace(num: &mut str) {
    let mut idx = num.len() - 1;
    unsafe {
        let bytes = num.as_bytes_mut();
        loop {
            if bytes[idx] == b'9' {
                bytes[idx] = b'0';
                idx -= 1;
            } else if bytes[idx] == b'.' {
                idx -= 1;
            } else {
                bytes[idx] += 1;
                break;
            }
        }
    }
}

/// Converts a positive finite binary float into a string representing a decimal
/// number dₘ⋯d₀.d₋₁⋯d₋ₚ where d ∈ [0..9] and p is the given number of
/// fractional digits.
/// The result may have an additional leading zero!
pub(super) fn bin_2_dec_fixed_point(f: f256, prec: usize) -> String {
    debug_assert!(f.is_finite());
    debug_assert!(f.is_sign_positive());
    let mut exp2 = f.exponent();
    let mut signif2 = f.significand();
    let ntz = signif2.trailing_zeros() as i32;
    let (is_less_than_one, is_int, (ip, fp)) = if exp2 >= -ntz {
        (false, true, (f, f256::ZERO))
    } else if exp2 < -(FRACTION_BITS as i32) {
        (true, false, (f256::ZERO, f))
    } else {
        (false, false, f.split())
    };
    let buf_len = if is_less_than_one {
        3 + prec
    } else {
        floor_log10_pow2(exp2 + FRACTION_BITS as i32) as usize + 3 + prec
    };
    let mut res = String::with_capacity(buf_len);
    // Preserve one char for carry
    res.push('0');
    // Integer part
    if is_less_than_one {
        res.push('0');
    } else {
        res.push_str(ip.to_string().as_str());
    }
    // Fractional part
    if prec > 0 {
        res.push('.');
    }
    let mut round = Round::Down;
    if is_int {
        res.push_str("0".repeat(prec).as_str());
    } else {
        exp2 = fp.exponent();
        signif2 = fp.significand();
        round = bin_fract_2_dec_str(signif2, exp2, prec, &mut res)
    }
    if round == Round::Up
        || (round == Round::ToEven && res.ends_with(['1', '3', '5', '7', '9']))
    {
        round_up_fixed_point_inplace(&mut res);
    }
    res
}

#[inline]
fn split_into_buf(buf: &mut String, s: &str) {
    buf.push_str(&s[..1]);
    buf.push('.');
    buf.push_str(&s[1..]);
}

fn bin_small_float_2_scientific(
    signif2: u256,
    exp2: i32,
    prec: usize,
    buf: &mut String,
) -> (Round, i32) {
    debug_assert!(exp2 > -(SIGNIFICAND_BITS as i32) && exp2 < 0);
    let mut exp10 = floor_log10_pow2(FRACTION_BITS as i32 + exp2);
    // Need to calculate the prec+1 left-most decimal digits of the number.
    // -237 < exp2 < 0
    // 0 <= exp10 < 71
    // k = prec - exp10
    // n = -exp2
    // signif10 = ⌊signif2 × 10ᵏ / 2ⁿ⌋, rounded tie to even
    let k = prec as i32 - exp10;
    let n = -exp2;
    // 0 <= prec <= 99 and 0 <= exp10 < 71 => -70 <= k <= 99
    let signif10 = if k >= 0 {
        // k >= 0 and 10ᵏ = 5ᵏ × 2ᵏ =>
        // ⌊signif2 × 10ᵏ / 2ⁿ⌋ = ⌊signif2 × 5ᵏ × 2ᵏ⁻ⁿ⌋
        let mut t = signif2.widening_mul(&get_power_of_five(k as u32));
        if k < n {
            // k < n => ⌊signif2 × 5ᵏ × 2ᵏ⁻ⁿ⌋ = ⌊signif2 × 5ᵏ / 2ⁿ⁻ᵏ⌋
            // 0 < n < 237 and 0 <= k < n => 0 < (n - k) < 237
            t.idiv_pow2((n - k) as u32);
        } else {
            // 0 < n < 237 and n <= k <= 99 => 0 < (k - n) < 99
            t <<= (k - n) as u32;
        }
        t
    } else {
        // k < 0 and 10ᵏ = 5ᵏ × 2ᵏ =>
        // ⌊signif2 × 10ᵏ / 2ⁿ⌋ = ⌊signif2 / (5⁻ᵏ × 2ⁿ⁻ᵏ)⌋
        // The value of k has been choosen so that the resulting decimal
        // significand is >= 1, thus the divident of the calculated
        // quotient must be greater than or equal to the divisor.
        // As signif2 < 2²³⁷ the same must hold for the divisor. Thus the
        // following shift can't overflow.
        let d = &get_power_of_five(-k as u32) << (n - k) as u32;
        let mut t = signif2.div_rounded(&d);
        u512::new(u256::ZERO, t)
    };
    let mut s = signif10.to_string();
    if s.len() > prec + 1 {
        // rounding overflow
        s = s[..s.len() - 1].to_string();
        exp10 += 1;
    }
    if prec == 0 {
        buf.push_str(&s);
    } else {
        split_into_buf(buf, &s);
    }
    (Round::Down, exp10)
}

fn bin_small_int_2_scientific(
    signif2: u256,
    exp2: i32,
    prec: usize,
    buf: &mut String,
) -> (Round, i32) {
    debug_assert!(exp2 >= 0 && exp2 <= (u512::BITS - SIGNIFICAND_BITS) as i32);
    let mut exp10 = floor_log10_pow2(SIGNIFICAND_BITS as i32 + exp2);
    // Need to calculate the prec+1 left-most decimal digits of the number.
    // 0 <= exp2 <= 275
    // 71 <= exp10 <= 154
    // k = exp10 - prec
    // n = exp2
    // signif10 = ⌊signif2 × 2ⁿ / 10ᵏ⌋
    let k = exp10 - prec as i32;
    let mut t = u512::new(u256::ZERO, signif2);
    t <<= exp2 as u32;
    let signif10 = t.div_pow10_rounded(k as u32);
    let s = signif10.to_string();
    if prec == 0 {
        buf.push_str(&s);
    } else {
        split_into_buf(buf, &s);
    }
    (Round::Down, exp10)
}

fn bin_large_int_2_scientific(
    signif2: u256,
    exp2: i32,
    prec: usize,
    buf: &mut String,
) -> (Round, i32) {
    debug_assert!(exp2 > (u512::BITS - SIGNIFICAND_BITS) as i32);
    let mut round = Round::Down;
    let (mut signif10, exp10) =
        DecNumRepr::shortest_from_bin_repr(signif2, exp2);
    // TODO: need u256::log10
    let k = floor_log10_pow2(signif10.msb() as i32) - prec as i32;
    if k > 0 {
        signif10 = signif10.div_pow10_rounded(k as u32);
    }
    let s = signif10.to_string();
    if prec == 0 {
        buf.push_str(&s);
    } else {
        split_into_buf(buf, &s);
    }
    (round, exp10)
}

fn bin_fract_2_scientific(
    signif2: u256,
    exp2: i32,
    prec: usize,
    buf: &mut String,
) -> (Round, i32) {
    debug_assert!(exp2 <= -(SIGNIFICAND_BITS as i32));
    let mut round = Round::Down;
    let segment_idx =
        (-exp2 - (SIGNIFICAND_BITS as i32)) as u32 / COMPRESSION_RATE;
    let (n_zero_chunks, segment_shift) = get_segment_params(segment_idx);
    debug_assert!(segment_shift > exp2);
    let mut exp10 = -((n_zero_chunks * CHUNK_SIZE) as i32);
    let shift = (segment_shift - exp2) as u32;
    let mut n_rem_digits = prec;
    let mut chunk_idx = 0_u32;
    let mut t = pow10_div_pow2(segment_idx, chunk_idx);
    let mut chunk = mul_shift_mod(&signif2, &t, shift);
    while chunk == 0 {
        exp10 -= CHUNK_SIZE as i32;
        chunk_idx += 1;
        assert!(
            chunk_idx < CHUNK_CUTOFF,
            "Internal limit for significant fractional digits exceeded."
        );
        t = pow10_div_pow2(segment_idx, chunk_idx);
        chunk = mul_shift_mod(&signif2, &t, shift);
    }
    let mut chunk_size = CHUNK_SIZE;
    let mut n_digits = min(chunk_size as usize, n_rem_digits);
    if buf.len() == 0 {
        let mut n = floor_log10(chunk);
        let d = 10_u64.pow(n);
        let i = chunk / d;
        buf.push_str(i.to_string().as_str());
        if n_rem_digits > 0 {
            buf.push('.');
        }
        exp10 -= (chunk_size - n) as i32;
        chunk %= d;
        chunk_size = n;
        n_digits = min(chunk_size as usize, n_rem_digits);
    }
    while n_digits < n_rem_digits {
        if n_digits > 0 {
            buf.push_str(format!("{:01$}", chunk, n_digits).as_str());
            n_rem_digits -= n_digits;
        }
        chunk_idx += 1;
        assert!(
            chunk_idx < CHUNK_CUTOFF,
            "Internal limit for significant fractional digits exceeded."
        );
        t = pow10_div_pow2(segment_idx, chunk_idx);
        chunk = mul_shift_mod(&signif2, &t, shift);
        chunk_size = CHUNK_SIZE;
        n_digits = min(chunk_size as usize, n_rem_digits);
    }
    // last chunk
    let d = 10_u64.pow(chunk_size - n_digits as u32);
    let rem = chunk % d;
    chunk /= d;
    let tie = d >> 1;
    if rem > tie {
        round = Round::Up;
    } else if rem == tie {
        // Need to check whether we really have a tie, i.e.
        // signif2 * 10 ^ (prec + 1) / 2 ^ -exp2 is an
        // integer. This is the case if the number of
        // trailing zeroes of the numerator is greater or
        // equal to -exp2.
        round =
            if (signif2.trailing_zeros() as i32) >= (-exp2 - prec as i32 - 1) {
                Round::ToEven
            } else {
                Round::Up
            };
    }
    if n_digits > 0 {
        buf.push_str(format!("{:01$}", chunk, n_rem_digits).as_str());
    }
    (round, exp10)
}

/// Round-Up the given scientific string representation of a decimal number.
#[allow(unsafe_code)]
#[inline]
fn round_up_scientific_inplace(num: &mut str) -> i32 {
    let mut carry = 0_i32;
    let mut idx = num.len() - 1;
    unsafe {
        let bytes = num.as_bytes_mut();
        loop {
            // First digit
            if idx == 0 && bytes[idx] == b'9' {
                bytes[idx] = b'1';
                carry = 1;
                break;
            } else if bytes[idx] == b'9' {
                bytes[idx] = b'0';
                idx -= 1;
            } else if bytes[idx] == b'.' {
                idx -= 1;
            } else {
                bytes[idx] += 1;
                break;
            }
        }
    }
    carry
}

/// Converts a positive finite binary float into a string representing a decimal
/// number d₀.d₋₁⋯d₋ₚEe where d ∈ [0..9], e ∈ [-78912..78913], E is the given
/// exponent marker and p is the given number of fractional digits.
pub(super) fn bin_2_dec_scientific(
    f: f256,
    exp_mark: char,
    prec: usize,
) -> String {
    debug_assert!(f.is_finite());
    debug_assert!(f.is_sign_positive());
    const SUBNORMAL_EXP_LOWER_BOUND: i32 = EMIN - FRACTION_BITS as i32;
    const SUBNORMAL_EXP_UPPER_BOUND: i32 = EMIN - 1;
    const NORMAL_EXP_LOWER_BOUND: i32 = EMIN;
    const FAST_LOWER_BOUND: i32 = -(FRACTION_BITS as i32);
    const FAST_LOWER_BOUND_MINUS_1: i32 = FAST_LOWER_BOUND - 1;
    const FAST_UPPER_BOUND: i32 = (u512::BITS - SIGNIFICAND_BITS) as i32;
    const FAST_UPPER_BOUND_PLUS_1: i32 = FAST_UPPER_BOUND + 1;
    const EXP_UPPER_BOUND: i32 = EMAX - FRACTION_BITS as i32;
    let mut exp2 = f.exponent();
    let mut signif2 = f.significand();
    let mut res = String::with_capacity(prec + 9);
    let mut round = Round::Down;
    let mut exp10 = 0_i32;
    match exp2 {
        // TODO: change the following ranges to exclusive upper bounds when
        //  feature(exclusive_range_pattern) got stable.
        FAST_LOWER_BOUND..=-1 => {
            // 1 <= f < 2²³⁶
            (round, exp10) =
                bin_small_float_2_scientific(signif2, exp2, prec, &mut res);
        }
        0..=FAST_UPPER_BOUND => {
            // 2²³⁶ <= f < 2⁴⁹¹
            (round, exp10) =
                bin_small_int_2_scientific(signif2, exp2, prec, &mut res);
        }
        NORMAL_EXP_LOWER_BOUND..=FAST_LOWER_BOUND_MINUS_1 => {
            // f256::MIN_POSITIVE <= f < 1
            (round, exp10) =
                bin_fract_2_scientific(signif2, exp2, prec, &mut res);
        }
        FAST_UPPER_BOUND_PLUS_1..=EXP_UPPER_BOUND => {
            // 2⁴⁹¹ <= f <= f256::MAX
            (round, exp10) =
                bin_large_int_2_scientific(signif2, exp2, prec, &mut res);
            // Need trailing zeroes?
            res.push_str("0".repeat(prec - min(prec, exp10 as usize)).as_str());
        }
        SUBNORMAL_EXP_LOWER_BOUND..=SUBNORMAL_EXP_UPPER_BOUND => {
            // f256::MIN_GT_ZERO <= f < MIN_POSITIVE
            // f is subnormal, adjust significand and exponent.
            let adj = SIGNIFICAND_BITS - signif2.msb();
            signif2 <<= adj;
            exp2 -= adj as i32;
            (round, exp10) =
                bin_fract_2_scientific(signif2, exp2, prec, &mut res);
        }
        _ => {
            unreachable!()
        }
    }
    if round == Round::Up
        || (round == Round::ToEven && res.ends_with(['1', '3', '5', '7', '9']))
    {
        exp10 += round_up_scientific_inplace(&mut res);
    }

    res.push(exp_mark);
    res.push_str(exp10.to_string().as_str());
    res
}

#[cfg(test)]
mod to_fixed_point_tests {
    use core::str::FromStr;
    use std::ops::Index;

    use super::*;

    #[test]
    fn test_one() {
        let f = f256::ONE;
        let s = bin_2_dec_fixed_point(f, 2);
        assert_eq!(s, "01.00");
    }

    #[test]
    fn test_one_half() {
        let f = f256::from_str("0.5").unwrap();
        let s = bin_2_dec_fixed_point(f, 0);
        assert_eq!(s, "00");
    }

    #[test]
    fn test_five_times_ten_pow_minus_twenty() {
        let f = f256::from_str("0.00000000000000000005").unwrap();
        let s = bin_2_dec_fixed_point(f, 19);
        assert_eq!(s, "00.0000000000000000001");
    }

    #[test]
    fn test_a_bit_less_ten() {
        let f = f256::from_str("9.9995").unwrap();
        let s = bin_2_dec_fixed_point(f, 3);
        assert_eq!(s, "10.000");
    }

    #[test]
    fn test_epsilon() {
        let f = f256::EPSILON;
        let s = bin_2_dec_fixed_point(f, 75);
        assert_eq!(
            s,
            "00.000000000000000000000000000000000000000000000000000000000\
            000000000000009056"
        );
    }

    #[test]
    fn test_min_gt_zero() {
        let f = f256::MIN_GT_ZERO;
        let s = bin_2_dec_fixed_point(f, 79004);
        assert!(s.starts_with("00.0"));
        assert_eq!(s[s.len() - 23..], "00224800708647703657297".to_string());
    }

    #[test]
    fn test_7e28() {
        let f = f256::from_str("7e28").unwrap();
        let s = bin_2_dec_fixed_point(f, 3);
        assert_eq!(s, "070000000000000000000000000000.000");
    }

    #[test]
    fn test_one_sixteenth() {
        let f = f256::from_str("0.0625").unwrap();
        let s = bin_2_dec_fixed_point(f, 23);
        assert_eq!(s, "00.06250000000000000000000");
    }
}

#[cfg(test)]
mod to_scientific_tests {
    use core::str::FromStr;
    use std::ops::Index;

    use super::*;

    #[test]
    fn test_one() {
        let f = f256::ONE;
        let s = bin_2_dec_scientific(f, 'e', 2);
        assert_eq!(s, "1.00e0");
    }

    #[test]
    fn test_one_half() {
        let f = f256::from_str("0.5").unwrap();
        let s = bin_2_dec_scientific(f, 'e', 0);
        assert_eq!(s, "5e-1");
    }

    #[test]
    fn test_two_and_a_half() {
        let f = f256::from_str("2.5").unwrap();
        let s = bin_2_dec_scientific(f, 'e', 0);
        assert_eq!(s, "2e0");
        let s = bin_2_dec_scientific(f, 'E', 3);
        assert_eq!(s, "2.500E0");
    }

    #[test]
    fn test_three_and_a_half() {
        let f = f256::from_str("3.5").unwrap();
        let s = bin_2_dec_scientific(f, 'e', 0);
        assert_eq!(s, "4e0");
        let s = bin_2_dec_scientific(f, 'E', 3);
        assert_eq!(s, "3.500E0");
    }

    #[test]
    fn test_one_sixteenth() {
        let f = f256::from_str("0.0625").unwrap();
        let s = bin_2_dec_scientific(f, 'e', 20);
        assert_eq!(s, "6.25000000000000000000e-2");
    }

    #[test]
    fn test_5_pow_4() {
        let f = f256::from_str("625").unwrap();
        let s = bin_2_dec_scientific(f, 'e', 3);
        assert_eq!(s, "6.250e2");
    }

    #[test]
    fn test_epsilon() {
        let f = f256::EPSILON;
        let s = bin_2_dec_scientific(f, 'E', 2);
        assert_eq!(s, "9.06E-72");
        let s = bin_2_dec_scientific(f, 'e', 4);
        assert_eq!(s, "9.0557e-72");
        let s = bin_2_dec_scientific(f, 'e', 6);
        assert_eq!(s, "9.055679e-72");
    }

    #[test]
    fn test_min_gt_zero() {
        let f = f256::MIN_GT_ZERO;
        let s = bin_2_dec_scientific(f, 'e', 20);
        assert_eq!(s, "2.24800708647703657297e-78984".to_string());
    }

    #[test]
    fn test_rounding_overflow() {
        let f = f256::from_str("97").unwrap();
        let s = bin_2_dec_scientific(f, 'e', 0);
        assert_eq!(s, "1e2");
        let f = f256::from_str("9999999999999999999997").unwrap();
        let s = bin_2_dec_scientific(f, 'e', 20);
        assert_eq!(s, "1.00000000000000000000e22");
    }
}
