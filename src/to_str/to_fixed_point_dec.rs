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
use crate::{f256, u256, u512, FRACTION_BITS, SIGNIFICAND_BITS};

const CHUNK_BASE: u64 = 10_u64.pow(CHUNK_SIZE);
const SHIFT: u32 = ADDITIONAL_BITS + COMPRESSION_RATE;

/// Calculate the segment index for POW2_DIV_POW10_TABLE from exponent
#[inline(always)]
fn calc_segment_idx(exp: u32) -> u32 {
    (exp + COMPRESSION_RATE - 1) / COMPRESSION_RATE
}

/// Calculate the number of chunks from positive exponent.
#[inline(always)]
fn calc_n_chunks(exp2: i32) -> u32 {
    debug_assert!(exp2 >= 0);
    ((floor_log10_pow2(exp2)) as u32 + COMPRESSION_RATE + CHUNK_SIZE)
        / CHUNK_SIZE
}

/// Calculate the minimal number of zero chunks from segment index.
#[inline(always)]
fn calc_n_zero_chunks(idx: u32) -> u32 {
    floor_log10_pow2((idx * COMPRESSION_RATE + SIGNIFICAND_BITS) as i32) as u32
        / CHUNK_SIZE
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

/// Round-up the given string representation of a decimal number.
#[allow(unsafe_code)]
#[inline]
fn round_up_inplace(num: &mut str) {
    let mut idx = num.len() - 1;
    unsafe {
        let bytes = num.as_bytes_mut();
        loop {
            if bytes[idx] == b'9' {
                bytes[idx] = b'0';
                idx -= 1;
            } else if bytes[idx] == b'.' {
                idx -= 1
            } else {
                bytes[idx] += 1;
                break;
            }
        }
    }
}

/// Converts a positive finite binary float into a string representing a decimal
/// number w × 10⁻ⁿ where n is the number of fractional digits.
/// The result may have an additional leading zero!
pub(super) fn bin_2_dec_str(f: f256, prec: usize) -> String {
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
    let mut round_up = false;
    if is_int {
        res.push_str("0".repeat(prec).as_str());
    } else {
        exp2 = fp.exponent();
        signif2 = fp.significand();
        let segment_idx =
            (-exp2 - SIGNIFICAND_BITS as i32) as u32 / COMPRESSION_RATE;
        let n_chunks = prec as u32 / CHUNK_SIZE + 1;
        let (n_zero_chunks, segment_shift) = get_segment_params(segment_idx);
        debug_assert!(segment_shift > exp2);
        let shift = (segment_shift - exp2) as u32;
        if n_chunks <= n_zero_chunks {
            res.push_str("0".repeat(prec).as_str());
        } else {
            if n_zero_chunks > 0 {
                res.push_str(
                    "0".repeat((n_zero_chunks * CHUNK_SIZE) as usize).as_str(),
                );
            }
            let n_signif_chunks = n_chunks - n_zero_chunks;
            assert!(
                n_signif_chunks <= CHUNK_CUTOFF,
                "Internal limit for significand digits exceeded."
            );
            for chunk_idx in 0..n_signif_chunks {
                let t = pow10_div_pow2(segment_idx, chunk_idx);
                let mut chunk = mul_shift_mod(&signif2, &t, shift);
                if chunk_idx < n_signif_chunks - 1 {
                    res.push_str(
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
                        round_up = true;
                    } else if rem == tie {
                        if (chunk & 1) == 1 ||
                            // Need to check whether we really have a tie, i.e.
                            // signif2 * 10 ^ (prec + 1) / 2 ^ -exp2 is an
                            // integer. This is the case if the number of
                            // trailing zeroes of the numerator is greater or
                            // equal to -exp2.
                            (signif2.trailing_zeros() as i32)
                                < (-exp2 - prec as i32 - 1)
                        {
                            round_up = true;
                        }
                    }
                    if n_digits > 0 {
                        res.push_str(
                            format!("{:01$}", chunk, n_digits as usize)
                                .as_str(),
                        );
                    }
                }
            }
        }
    }
    if round_up {
        round_up_inplace(&mut res);
    }
    res
}

#[cfg(test)]
mod tests {
    use core::str::FromStr;
    use std::ops::Index;

    use super::*;

    #[test]
    fn test_one() {
        let f = f256::ONE;
        let s = bin_2_dec_str(f, 2);
        assert_eq!(s, "01.00");
    }

    #[test]
    fn test_one_half() {
        let f = f256::from_str("0.5").unwrap();
        let s = bin_2_dec_str(f, 0);
        assert_eq!(s, "00");
    }

    #[test]
    fn test_five_times_ten_pow_minus_twenty() {
        let f = f256::from_str("0.00000000000000000005").unwrap();
        let s = bin_2_dec_str(f, 19);
        assert_eq!(s, "00.0000000000000000001");
    }

    #[test]
    fn test_a_bit_less_ten() {
        let f = f256::from_str("9.9995").unwrap();
        let s = bin_2_dec_str(f, 3);
        assert_eq!(s, "10.000");
    }

    #[test]
    fn test_epsilon() {
        let f = f256::EPSILON;
        let s = bin_2_dec_str(f, 75);
        assert_eq!(
            s,
            "00.000000000000000000000000000000000000000000000000000000000\
            000000000000009056"
        );
    }

    #[test]
    fn test_min_gt_zero() {
        let f = f256::MIN_GT_ZERO;
        let s = bin_2_dec_str(f, 79004);
        assert!(s.starts_with("00.0"));
        assert_eq!(s[s.len() - 23..], "00224800708647703657297".to_string());
    }

    #[test]
    fn test_7e28() {
        let f = f256::from_str("7e28").unwrap();
        let s = bin_2_dec_str(f, 3);
        assert_eq!(s, "070000000000000000000000000000.000");
    }

    #[test]
    fn test_one_sixteenth() {
        let f = f256::from_str("0.0625").unwrap();
        // assert_eq!(f.fract(), f256::encode(0, -4, u256::new(0, 1)));
        let s = bin_2_dec_str(f, 23);
        assert_eq!(s, "00.06250000000000000000000");
    }
}
