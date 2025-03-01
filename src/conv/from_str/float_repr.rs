// ---------------------------------------------------------------------------
// Copyright:   (c) 2021 ff. Michael Amrhein (michael@adrhinum.de)
// License:     This program is part of a larger application. For license
//              details please read the file LICENSE.TXT provided together
//              with the application.
// ---------------------------------------------------------------------------
// $Source$
// $Revision$

use core::{
    cmp::{max, min},
    fmt::Display,
};

use super::common::{
    chunk_contains_7_digits_and_a_dot_at, chunk_contains_8_digits,
    chunk_to_u64, AsciiNumLit,
};
use crate::U256;

// Remove the byte at position p by shifting the bytes left from it right.
const fn eliminate_byte_from_chunk(k: u64, p: u32) -> u64 {
    match p {
        0 => (k << 8) >> 8,
        1..=6 => {
            // bytes right from byte p
            (k & (u64::MAX >> (8 * (p + 1))))
                // bytes left from byte p
                | ((k & (u64::MAX << (8 * (8 - p)))) >> 8)
        }
        _ => k >> 8,
    }
}

// Records the final parsing result in case of a valid number, i.e. the sign,
// the exponent and the value of the (maybe partial) significand together with
// an indicator that the limit of digits fitting into an u256 has been
// exceeded.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(super) struct DecNumRepr {
    pub(super) sign: u32,
    pub(super) exponent: i32,
    pub(super) significand: U256,
    pub(super) signif_truncated: bool,
}

// Records the final parsing result.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum FloatRepr {
    Empty,
    Invalid,
    Nan,
    Inf(u32),
    Number(DecNumRepr),
}

// Records the first (atmost MAX_N_DIGITS) significant digits:
// Max 9 chunks of 8 digits (each converted to an u64), a related counter plus
// the remaining digits as u64 together with the number of these digits.
#[derive(Clone, Copy, Debug, Default)]
struct PartialSignif {
    chunks: [u64; 9],
    rem: u64,
    n_chunks: usize,
    n_rem_digits: usize,
}

impl PartialSignif {
    const MAX_N_DIGITS: usize = 77;

    const fn max_add_digits(&self) -> usize {
        Self::MAX_N_DIGITS - self.n_chunks * 8 - self.n_rem_digits
    }

    fn add_chunk(&mut self, value: u64) {
        self.chunks[self.n_chunks] = value;
        self.n_chunks += 1;
    }

    const fn n_digits(&self) -> usize {
        self.n_chunks * 8 + self.n_rem_digits
    }

    fn normalize(&mut self) -> usize {
        let mut n_trailing_zeroes = 0_usize;
        if self.rem == 0 && self.n_chunks > 0 {
            n_trailing_zeroes += self.n_rem_digits;
            while self.n_chunks > 1 && self.chunks[self.n_chunks - 1] == 0 {
                n_trailing_zeroes += 8;
                self.n_chunks -= 1;
            }
            self.rem = self.chunks[self.n_chunks - 1];
            self.n_rem_digits = 8;
            self.n_chunks -= 1;
        }
        while self.n_rem_digits > 0 && (self.rem % 10) == 0 {
            self.rem /= 10;
            self.n_rem_digits -= 1;
            n_trailing_zeroes += 1;
        }
        n_trailing_zeroes
    }

    #[allow(clippy::cast_possible_truncation)]
    fn significand(&self) -> U256 {
        const CHUNK_BASE: U256 = U256::new(0_u128, 100000000_u128);
        let mut signif = U256::default();
        for i in 0..self.n_chunks {
            signif *= &CHUNK_BASE;
            signif += &(self.chunks[i] as u128);
        }
        signif *= &10_u128.pow(self.n_rem_digits as u32);
        signif += &(self.rem as u128);
        signif
    }
}

impl FloatRepr {
    /// Convert the leading sequence of decimal digits in `lit` (if any) into
    /// an int and accumulate it into `partial_signif`.
    #[allow(unsafe_code)]
    #[allow(clippy::cognitive_complexity)]
    fn read_significand(lit: &mut AsciiNumLit) -> PartialSignif {
        let mut partial_signif = PartialSignif::default();
        lit.state.start_pos_signif = lit.len();
        let max_n_digits = PartialSignif::MAX_N_DIGITS;
        let mut n_digits = 0_usize;
        // First, try chunks of 8 digits
        let limit = max_n_digits.saturating_sub(8);
        while n_digits <= limit {
            if let Some(mut k) = lit.read_u64() {
                if chunk_contains_8_digits(k) {
                    partial_signif.add_chunk(chunk_to_u64(k));
                    n_digits += 8;
                    // SAFETY: safe because of call to lit.read_u64 above
                    unsafe {
                        lit.skip_n(8);
                    }
                } else if let Some(p) =
                    chunk_contains_7_digits_and_a_dot_at(k)
                {
                    if lit.state.pos_radix_point.is_some() {
                        // Double radix point
                        lit.state.invalid = true;
                        return partial_signif;
                    }
                    // The index p points to the b'.' in chunk k in
                    // little-endian order while lit.len() is based on the
                    // big-endian order of the byte array. Therefore we have
                    // to subtract (7 -p) here.
                    lit.state.pos_radix_point =
                        Some(lit.len() - (7 - p) as usize);
                    // SAFETY: safe because of call to lit.read_u64 above
                    unsafe {
                        lit.skip_n(8);
                    }
                    if lit.first_is_digit() {
                        // Assemble 8-digit chunk
                        // SAFETY: unwrap is safe here because of call to
                        // lit.first above.
                        let d = *lit.first().unwrap();
                        // SAFETY: dito.
                        unsafe {
                            lit.skip_1();
                        }
                        // The bytes in chunk k are in little endian order!
                        // k <- d << 56 | k[..p] >> 8 | k[p+1..]
                        k = eliminate_byte_from_chunk(k, p);
                        k |= (d as u64) << 56;
                        partial_signif.add_chunk(chunk_to_u64(k));
                        n_digits += 8;
                    } else {
                        // No more digits
                        // k <- k[..p] | k[p+1..] << 8
                        k = eliminate_byte_from_chunk(k, p) << 8;
                        partial_signif.rem = chunk_to_u64(k);
                        partial_signif.n_rem_digits = 7;
                        n_digits += 7;
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        // Read single digits
        while let Some(c) = lit.first() {
            if n_digits < max_n_digits {
                let d = c.wrapping_sub(b'0');
                if d < 10 {
                    partial_signif.rem *= 10;
                    partial_signif.rem += d as u64;
                    partial_signif.n_rem_digits += 1;
                    n_digits += 1;
                } else if *c == b'.' {
                    if lit.state.pos_radix_point.is_some() {
                        // Double radix point
                        lit.state.invalid = true;
                        return partial_signif;
                    }
                    lit.state.pos_radix_point = Some(lit.len());
                } else {
                    break;
                }
                // SAFETY: safe because of call to lit.first above
                unsafe {
                    lit.skip_1();
                }
            } else {
                break;
            }
        }
        lit.state.end_pos_signif = lit.len();
        // Handle remaining digits
        let mut n_non_zero_digits = 0_usize;
        while let Some(c) = lit.first() {
            if *c >= b'1' && *c <= b'9' {
                n_non_zero_digits += 1;
            } else if *c == b'.' {
                if lit.state.pos_radix_point.is_some() {
                    // Double radix point
                    lit.state.invalid = true;
                    return partial_signif;
                }
                lit.state.pos_radix_point = Some(lit.len());
            } else if *c != b'0' {
                break;
            }
            // SAFETY: safe because of call to lit.first above
            unsafe {
                lit.skip_1();
            }
        }
        if n_non_zero_digits > 0 {
            lit.state.signif_truncated = true;
        }
        partial_signif
    }

    #[allow(unsafe_code)]
    fn parse_special(lit: &mut AsciiNumLit, sign: u32) -> Self {
        if lit.eq_ignore_ascii_case(b"nan") {
            Self::Nan
        } else if lit.eq_ignore_ascii_case(b"inf")
            || lit.eq_ignore_ascii_case(b"infinity")
        {
            Self::Inf(sign)
        } else {
            Self::Invalid
        }
    }

    #[allow(unsafe_code)]
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_possible_wrap)]
    pub(crate) fn from_str(s: &str) -> Self {
        let mut lit = AsciiNumLit::new(s.as_ref());

        if lit.is_empty() {
            return Self::Empty;
        }

        let sign = lit.get_sign();

        if lit.is_empty() {
            return Self::Invalid;
        }
        let start_pos = lit.len();

        lit.skip_leading_zeroes(true);
        if lit.is_empty() {
            // There must have been atleast one zero.
            return Self::Number(DecNumRepr::default());
        }

        // Parse significant digits.
        let mut partial_signif = Self::read_significand(&mut lit);

        // Check state.
        if lit.state.invalid {
            return Self::Invalid;
        }

        // If there are no digits, check for special values.
        if start_pos == lit.len() {
            return Self::parse_special(&mut lit, sign);
        }

        // Set implicit radix point if no one was detected.
        if lit.state.pos_radix_point.is_none() {
            lit.state.pos_radix_point = Some(lit.len());
        }

        // Check for explicit exponent
        let mut exponent = match lit.parse_exponent() {
            Some(exp) => exp,
            None => {
                return Self::Invalid;
            }
        };

        // Check bounds.
        if !lit.is_empty() {
            return Self::Invalid;
        }

        // Adjust exponent by truncated integer digits and fractional digits.
        let pos_radix_point =
            // Can't be None here.
            lit.state.pos_radix_point.unwrap();
        let n_int_digits =
            lit.state.start_pos_signif.saturating_sub(pos_radix_point);
        let n_truncated_int_digits =
            n_int_digits.saturating_sub(PartialSignif::MAX_N_DIGITS);
        exponent += n_truncated_int_digits as i32;
        let n_frac_digits =
            pos_radix_point.saturating_sub(lit.state.end_pos_signif + 1);
        exponent -= n_frac_digits as i32;

        // Get normalized significand and adjust exponent
        let n_trailing_zeroes = partial_signif.normalize();
        exponent += n_trailing_zeroes as i32;
        let significand = partial_signif.significand();
        Self::Number(DecNumRepr {
            sign,
            exponent,
            significand,
            signif_truncated: lit.state.signif_truncated,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_zero_digits_with_dot() {
        let s = "000.0000000000000000";
        let r = FloatRepr::from_str(s);
        assert_eq!(r, FloatRepr::Number(DecNumRepr::default()));
    }

    #[test]
    fn parse_digits_with_trailing_zeroes_before_dot() {
        let s = "180000.0000000000000000000000000000000000000000000000000\
            000000000000000000000000000000000000000000000000000000000000000000";
        let r = FloatRepr::from_str(s);
        assert_eq!(
            r,
            FloatRepr::Number(DecNumRepr {
                sign: 0,
                exponent: 4,
                significand: U256::new(0, 18),
                signif_truncated: false
            })
        );
    }

    #[test]
    fn parse_frac_only_with_trailing_zeroes() {
        let s = "-.000000000000000010000000000000000000000000000000000000\
            00000000000000000000000000000000000000000000000000000000000000000";
        let r = FloatRepr::from_str(s);
        assert_eq!(
            r,
            FloatRepr::Number(DecNumRepr {
                sign: 1,
                exponent: -17,
                significand: U256::new(0, 1),
                signif_truncated: false
            })
        );
    }

    #[test]
    fn parse_nonzero_digits_with_dot_and_trailing_zeroes_and_exp() {
        let s = "-7.62939453125000000000000000000000000000000000000000000\
            0000000000000000000000000000000000000000000000000000000000000000000\
            00e-06";
        let r = FloatRepr::from_str(s);
        assert_eq!(
            r,
            FloatRepr::Number(DecNumRepr {
                sign: 1,
                significand: U256::new(0, 762939453125),
                exponent: -17,
                signif_truncated: false
            })
        );
    }

    #[test]
    fn parse_digits_with_more_than_77_int_digits() {
        let s = "-1000000000000000000000000000000000000000000000000000000\
            000000000000000000000000000000000000000000000000000000000000.0e-06";
        let r = FloatRepr::from_str(s);
        assert_eq!(
            r,
            FloatRepr::Number(DecNumRepr {
                sign: 1,
                significand: U256::new(0, 1),
                exponent: 108,
                signif_truncated: false
            })
        );
    }

    #[test]
    fn test_large_signif_large_neg_exp() {
        let s = "-441.249009748590979791323783709646682894752724672748600\
                542581589000e-78928";
        let r = FloatRepr::from_str(s);
        assert_eq!(
            r,
            FloatRepr::Number(DecNumRepr {
                sign: 1,
                significand: U256::new(
                    1296714295663492914563767,
                    83422945940683308502053253308490666837
                ),
                exponent: -78988,
                signif_truncated: false
            })
        );
    }
}
